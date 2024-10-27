use crate::display::request::{self, *};
use crate::display::error::Error;
use crate::display::{Atom, Visual, Roots, Stream, TryClone};
use crate::display::xid;
use crate::proto::*;

use std::io::{Read, Write};


/// a builder for a list of values known as `LISTofVALUE` in proto.pdf
pub struct ValuesBuilder<T: ValueMask> {
    values: Vec<T>,
    request: Vec<u8>,
    mask: u32,
}

impl<T> ValuesBuilder<T> where T: ValueMask {

    /// create a new values builder with specified values
    pub fn new(values: Vec<T>) -> ValuesBuilder<T> {
        ValuesBuilder {
            values,
            request: Vec::new(),
            mask: 0,
        }
    }

    pub(crate) fn len(&self) -> u16 { self.values.len() as u16 }

    pub(crate) fn build(&mut self) -> Vec<u8> {
        self.values.sort_by(|a, b| a.mask().partial_cmp(&b.mask()).unwrap());

        for value in &self.values {
            self.mask |= value.mask();

            self.request.extend(value.encode());

            assert!(self.request.len() % 4 == 0);
        }

        self.request.clone()
    }
}

pub trait ValueMask {
    fn mask(&self) -> u32;

    fn encode(&self) -> Vec<u8>;
}

/// representing value in a configure window request
pub enum ConfigureValue<T: Send + Sync + Read + Write + TryClone> {
    X(u16),
    Y(u16),
    Width(u16),
    Height(u16),
    Border(u16),
    Sibling(Window<T>),
    StackMode(StackMode),
}

impl<T> ValueMask for ConfigureValue<T> where T: Send + Sync + Read + Write + TryClone {
    fn mask(&self) -> u32 {
        match self {
            ConfigureValue::X(_) => 0x1,
            ConfigureValue::Y(_) => 0x2,
            ConfigureValue::Width(_) => 0x4,
            ConfigureValue::Height(_) => 0x8,
            ConfigureValue::Border(_) => 0x10,
            ConfigureValue::Sibling(_) => 0x20,
            ConfigureValue::StackMode(_) => 0x40,
        }
    }

    fn encode(&self) -> Vec<u8> {
        match self {
            ConfigureValue::X(value)
                | ConfigureValue::Y(value)
                | ConfigureValue::Width(value)
                | ConfigureValue::Height(value)
                | ConfigureValue::Border(value) => request::encode(&(*value as u32)).to_vec(),

            ConfigureValue::Sibling(window) => request::encode(&window.id()).to_vec(),

            ConfigureValue::StackMode(stack_mode) => request::encode(&(*stack_mode as u32)).to_vec(),
        }
    }
}


#[derive(Clone)]
pub enum WindowValue {
    BgPixmap(u32),
    BgPixel(u32),
    BorderPixmap(u32),
    BorderPixel(u32),
    BitGravity(Gravity),
    WinGravity(Gravity),
    BackingStore(BackingStore),
    BackingPlane(u32),
    BackingPixel(u32),
    OverrideRedirect(bool),
    SaveUnder(bool),
    EventMask(Vec<EventMask>),
    DoNotPropogateMask(Vec<EventMask>),
    Colormap(u32),
    Cursor(Cursor),
}

impl WindowValue {
    fn mask(&self, masks: &[EventMask]) -> u32 {
        masks.iter()
            .map(|event_mask| *event_mask as u32)
            .fold(0, |acc, x| acc | x)
    }
}

impl ValueMask for WindowValue {
    fn mask(&self) -> u32 {
        match self {
            WindowValue::BgPixmap(_) => 0x00000001,
            WindowValue::BgPixel(_) => 0x00000002,
            WindowValue::BorderPixmap(_) => 0x00000004,
            WindowValue::BorderPixel(_) => 0x00000008,
            WindowValue::BitGravity(_) => 0x00000010,
            WindowValue::WinGravity(_) => 0x00000020,
            WindowValue::BackingStore(_) => 0x00000040,
            WindowValue::BackingPlane(_) => 0x00000080,
            WindowValue::BackingPixel(_) => 0x00000100,
            WindowValue::OverrideRedirect(_) => 0x00000200,
            WindowValue::SaveUnder(_) => 0x00000400,
            WindowValue::EventMask(_) => 0x00000800,
            WindowValue::DoNotPropogateMask(_) => 0x00001000,
            WindowValue::Colormap(_) => 0x00002000,
            WindowValue::Cursor(_) => 0x00004000,
        }
    }

    // TODO: there is something wrong here lol, the other one works tho lol
    fn encode(&self) -> Vec<u8> {
        match self {
            WindowValue::BgPixmap(value)
                | WindowValue::BgPixel(value)
                | WindowValue::BorderPixmap(value)
                | WindowValue::BorderPixel(value)
                | WindowValue::BackingPlane(value)
                | WindowValue::BackingPixel(value)
                | WindowValue::Colormap(value) => request::encode(&(*value as u32)).to_vec(),

            WindowValue::BitGravity(gravity)
                | WindowValue::WinGravity(gravity) => request::encode(&(*gravity as u32)).to_vec(),

            WindowValue::OverrideRedirect(value)
                | WindowValue::SaveUnder(value) => request::encode(&(*value as u32)).to_vec(),

            WindowValue::EventMask(masks)
                | WindowValue::DoNotPropogateMask(masks)=> request::encode(&self.mask(masks)).to_vec(),

            WindowValue::Cursor(cursor) => request::encode(&(*cursor as u32)).to_vec(),

            WindowValue::BackingStore(store) => request::encode(&(*store as u32)).to_vec(),
        }
    }
}

pub struct WindowArguments {
    pub depth: u8,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub class: WindowClass,
    pub visual: Visual,
    pub values: ValuesBuilder<WindowValue>,
}

pub enum WindowKind {
    Window,
    SubWindows,
}

impl WindowKind {
    fn encode(&self, subwindows: u8, window: u8) -> u8 {
        match self {
            WindowKind::Window => window,
            WindowKind::SubWindows => subwindows,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PropFormat {
    Format8 = 8,
    Format16 = 16,
    Format32 = 32,
}

impl PropFormat {
    pub fn encode(&self, len: usize) -> u32 {
        match self {
            PropFormat::Format8 => len as u32,
            PropFormat::Format16 => len as u32 / 2,
            PropFormat::Format32 => len as u32 / 4,
        }
    }
}

pub enum PropMode {
    Replace = 0,
    Prepend = 1,
    Append = 2,
}

pub struct Window<T: Send + Sync + Read + Write + TryClone> {
    stream: Stream<T>,
    replies: Queue<Reply>,
    sequence: SequenceManager,
    visual: Visual,
    depth: u8,
    id: u32,
}

impl<T> TryClone for Window<T> where T: Send + Sync + Read + Write + TryClone {
    fn try_clone(&self) -> Result<Box<Self>, Error> {
        Ok(Box::new(Window {
            stream: self.stream.try_clone()?,
            replies: self.replies.clone(),
            sequence: self.sequence.clone(),
            visual: self.visual.clone(),
            depth: self.depth,
            id: self.id,
        }))
    }
}

impl<T> Window<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, replies: Queue<Reply>, sequence: SequenceManager, visual: Visual, depth: u8, id: u32) -> Window<T> {
        Window {
            stream,
            replies,
            sequence,
            visual,
            depth,
            id,
        }
    }

    pub(crate) fn from_id(
        mut stream: Stream<T>,
        mut replies: Queue<Reply>,
        mut sequence: SequenceManager,
        roots: Roots,
        id: u32
    ) -> Result<Window<T>, Error> {
        sequence.append(ReplyKind::GetWindowAttributes)?;

        let screen = roots.first()?;

        stream.send_encode(GetWindowAttributes {
            opcode: Opcode::GET_WINDOW_ATTRIBUTES,
            pad0: 0,
            length: 2,
            wid: id,
        })?;

        match replies.wait()? {
            Reply::GetWindowAttributes(response) => {
                Ok(Window {
                    stream,
                    replies,
                    sequence,
                    visual: roots.visual_from_id(response.visual)?,
                    depth: screen.response.root_depth,
                    id,
                })
            },
            _ => unreachable!(),
        }
    }

    /// send an event to the window
    pub fn send_event(&mut self, event: Event, event_mask: Vec<EventMask>, propogate: bool) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(SendEvent {
            opcode: Opcode::SEND_EVENT,
            propogate: propogate.then(|| 1).unwrap_or(0),
            length: 11,
            destination: self.id(),
            event_mask: event_mask.iter().fold(0, |acc, mask| acc | *mask as u32),
        })?;

        self.stream.send(Into::<Vec<u8>>::into(event).as_slice())?;

        self.replies.poll_error()
    }

    /// get the window attributes
    pub fn get_window_attributes(&mut self) -> Result<GetWindowAttributesResponse, Error> {
        self.sequence.append(ReplyKind::GetWindowAttributes)?;

        self.stream.send_encode(GetWindowAttributes {
            opcode: Opcode::GET_WINDOW_ATTRIBUTES,
            pad0: 0,
            length: 2,
            wid: self.id(),
        })?;

        match self.replies.wait()? {
            Reply::GetWindowAttributes(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    /// get the geometry of the window
    pub fn get_geometry(&mut self) -> Result<GetGeometryResponse, Error> {
        self.sequence.append(ReplyKind::GetGeometry)?;

        self.stream.send_encode(GetGeometry {
            opcode: Opcode::GET_GEOMETRY,
            pad0: 0,
            length: 2,
            window: self.id(),
        })?;

        match self.replies.wait()? {
            Reply::GetGeometry(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    // TODO: un-hardcode current-time

    /// set the window to the selection owner
    pub fn set_selection_owner(&mut self, selection: Atom) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(SetSelectionOwner {
            opcode: Opcode::SET_SELECTION_OWNER,
            pad0: 0,
            length: 4,
            owner: self.id(),
            selection: selection.id(),
            time: 0,
        })?;

        self.replies.poll_error()
    }

    // TODO: un-hardcode current-time

    /// sends a selection request to the owner
    pub fn convert_selection(&mut self, selection: Atom, target: Atom, property: Atom) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(ConvertSelection {
            opcode: Opcode::CONVERT_SELECTION,
            pad0: 0,
            length: 6,
            requestor: self.id(),
            selection: selection.id(),
            target: target.id(),
            property: property.id(),
            time: 0,
        })?;

        self.replies.poll_error()
    }

    fn generic_window(&mut self, opcode: u8, length: u16) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(GenericWindow {
            opcode,
            pad0: 0,
            length,
            wid: self.id(),
        })?;

        self.replies.poll_error()
    }

    /// window id
    pub fn id(&self) -> u32 { self.id }

    /// window depth
    pub fn depth(&self) -> u8 { self.depth }

    /// visual assigned to the window
    pub fn visual(&self) -> Visual { self.visual.clone() }

    /// create a child window with provided window arguments
    pub fn create_window(&mut self, mut window: WindowArguments) -> Result<Window<T>, Error> {
        self.sequence.skip();

        let window_values_request = window.values.build();
        let wid = xid::next()?;

        self.stream.send_encode(CreateWindow {
            opcode: Opcode::CREATE_WINDOW,
            depth: window.depth,
            length: 8 + window.values.len(),
            wid,
            parent: self.id(),
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            border_width: window.border_width,
            class: window.class as u16,
            visual: window.visual.id,
            value_mask: window.values.mask,
        })?;

        self.stream.send(&window_values_request)?;

        self.replies.poll_error()?;

        Ok(Window::new(self.stream.try_clone()?, self.replies.clone(), self.sequence.clone(), window.visual, window.depth, wid))
    }

    /// kill the window
    pub fn kill(&mut self) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(KillClient {
            opcode: Opcode::KILL_CLIENT,
            pad0: 0,
            length: 2,
            resource: self.id(),
        })?;

        self.replies.poll_error()
    }

    /// sets the current input focus to the window
    pub fn set_input_focus(&mut self, revert_to: RevertTo) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(SetInputFocus {
            opcode: Opcode::SET_INPUT_FOCUS,
            revert_to: revert_to as u8,
            length: 3,
            focus: self.id(),
            time: 0,
        })?;

        self.replies.poll_error()
    }

    /// change the attributes of a window
    pub fn change_attributes(&mut self, mut values: ValuesBuilder<WindowValue>) -> Result<(), Error> {
        self.sequence.skip();

        let request = values.build();

        self.stream.send_encode(ChangeWindowAttributes {
            opcode: Opcode::CHANGE_WINDOW_ATTRIBUTES,
            pad0: 0,
            length: values.len() + 3,
            wid: self.id(),
            mask: values.mask,
        })?;

        self.stream.send(&request)?;

        self.replies.poll_error()
    }

    /// configure the window
    pub fn configure(&mut self, mut values: ValuesBuilder<ConfigureValue<T>>) -> Result<(), Error> {
        self.sequence.skip();

        let request = values.build();

        self.stream.send_encode(ConfigureWindow {
            opcode: Opcode::CONFIGURE_WINDOW,
            pad0: 0,
            length: values.len() + 3,
            wid: self.id(),
            mask: values.mask as u16,
            pad1: 0,
        })?;

        self.stream.send(&request)?;

        self.replies.poll_error()
    }

    /// set the border of a window to a pixel
    pub fn set_border_pixel(&mut self, pixel: u32) -> Result<(), Error> {
        self.change_attributes(ValuesBuilder::new(vec![WindowValue::BorderPixel(pixel)]))
    }

    /// set the border width of a window
    pub fn set_border_width(&mut self, width: u16) -> Result<(), Error> {
        self.configure(ValuesBuilder::new(vec![ConfigureValue::Border(width)]))
    }

    /// move a window, this is a fancy wrapper for configure
    pub fn mov(&mut self, x: u16, y: u16) -> Result<(), Error> {
        self.configure(ValuesBuilder::new(vec![ConfigureValue::X(x), ConfigureValue::Y(y)]))
    }

    /// resize a window, this is a fancy wrapper for configure
    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), Error> {
        self.configure(ValuesBuilder::new(vec![ConfigureValue::Width(width), ConfigureValue::Height(height)]))
    }

    /// move and resize a window, this is a fancy wrapper for configure
    pub fn mov_resize(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<(), Error> {
        self.mov(x, y)?;

        self.resize(width, height)
    }

    /// choose the events you want to recieve
    pub fn select_input(&mut self, events: &[EventMask]) -> Result<(), Error> {
        self.change_attributes(ValuesBuilder::new(vec![WindowValue::EventMask(events.to_vec())]))
    }

    /// raise the window to the top of the stack
    pub fn raise(&mut self) -> Result<(), Error> {
        self.configure(ValuesBuilder::new(vec![ConfigureValue::StackMode(StackMode::Above)]))
    }

    /// lower the window to the bottom of the stack
    pub fn lower(&mut self) -> Result<(), Error> {
        self.configure(ValuesBuilder::new(vec![ConfigureValue::StackMode(StackMode::Below)]))
    }

    /// become the child of a parent window
    pub fn reparent(&mut self, parent: Window<T>, x: u16, y: u16) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(ReparentWindow {
            opcode: Opcode::REPARENT_WINDOW,
            pad0: 0,
            length: 4,
            window: self.id(),
            parent: parent.id(),
            x,
            y,
        })?;

        self.replies.poll_error()
    }

    /// destroy the current window object
    pub fn destroy(mut self, kind: WindowKind) -> Result<(), Error> {
        self.generic_window(kind.encode(Opcode::DESTROY_SUBWINDOWS, Opcode::DESTROY_WINDOW), 2)
    }

    /// map the window onto the screen
    pub fn map(&mut self, kind: WindowKind) -> Result<(), Error> {
        self.generic_window(kind.encode(Opcode::MAP_SUBWINDOWS, Opcode::MAP_WINDOW), 2)
    }

    /// unmap the window
    pub fn unmap(&mut self, kind: WindowKind) -> Result<(), Error> {
        self.generic_window(kind.encode(Opcode::UNMAP_SUBWINDOWS, Opcode::UNMAP_WINDOW), 2)
    }

    /// change a property of a window
    pub fn change_property(
        &mut self,
        property: Atom,
        type_: Atom,
        format: PropFormat,
        mode: PropMode,
        data: &[u8]
    ) -> Result<(), Error> {
        self.sequence.skip();

        let request = ChangeProperty {
            opcode: Opcode::CHANGE_PROPERTY,
            mode: mode as u8,
            length: 6 + (data.len() as u16 + request::pad(data.len()) as u16) / 4,
            window: self.id(),
            property: property.id(),
            type_: type_.id(),
            format: format as u8,
            pad0: [0; 3],
            data_len: format.encode(data.len()),
        };

        self.stream.send(request::encode(&request))?;

        self.stream.send_pad(data)?;

        self.replies.poll_error()
    }

    /// delete a property from a window
    pub fn delete_property(&mut self, property: Atom) -> Result<(), Error> {
        self.generic_window(Opcode::DELETE_PROPERTY, 3)?;

        self.stream.send_encode(property.id())?;

        self.replies.poll_error()
    }

    /// get the value of a property from a window
    pub fn get_property(&mut self, property: Atom, type_: Atom, delete: bool) -> Result<(Vec<u8>, Atom), Error> {
        self.sequence.append(ReplyKind::GetProperty)?;

        self.stream.send_encode(GetProperty {
            opcode: Opcode::GET_PROPERTY,
            delete: delete.then(|| 1).unwrap_or(0),
            length: 6,
            window: self.id(),
            property: property.id(),
            type_: type_.id(),
            long_offset: u32::MIN,
            long_length: u16::MAX as u32,
        })?;

        match self.replies.wait()? {
            Reply::GetProperty { type_, value } => Ok((value, type_)),
            _ => unreachable!(),
        }
    }

    /// get info about the pointer such as position
    pub fn query_pointer(&mut self) -> Result<QueryPointerResponse, Error> {
        self.sequence.append(ReplyKind::QueryPointer)?;

        self.stream.send_encode(QueryPointer {
            opcode: Opcode::QUERY_POINTER,
            pad0: 0,
            length: 2,
            wid: self.id(),
        })?;

        match self.replies.wait()? {
            Reply::QueryPointer(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    /// grab a key from the window,
    /// buttons are not valid modifiers
    pub fn grab_key(
        &mut self,
        modifiers: Vec<KeyMask>,
        keycode: u8,
        pointer_mode: PointerMode,
        keyboard_mode: KeyboardMode,
        owner_events: bool,
    ) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(GrabKey {
            opcode: Opcode::GRAB_KEY,
            owner_events: owner_events.then(|| 1).unwrap_or(0),
            length: 4,
            grab_window: self.id(),
            modifiers: modifiers.iter().fold(0, |acc, modifier| acc | *modifier as u16),
            key: keycode,
            pointer_mode: pointer_mode as u8,
            keyboard_mode: keyboard_mode as u8,
            pad0: [0u8; 3],
        })?;

        self.replies.poll_error()
    }

    /// grab a button from the window,
    /// buttons are not valid modifiers
    pub fn grab_button(
        &mut self,
        button: Button,
        modifiers: Vec<KeyMask>,
        event_mask: Vec<EventMask>,
        cursor: Cursor,
        pointer_mode: PointerMode,
        keyboard_mode: KeyboardMode,
        owner_events: bool,
        confine_to: u32,
    ) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(GrabButton {
            opcode: Opcode::GRAB_BUTTON,
            owner_events: owner_events.then(|| 1).unwrap_or(0),
            length: 6,
            grab_window: self.id(),
            event_mask: event_mask.iter().fold(0, |acc, mask| acc | *mask as u16),
            pointer_mode: pointer_mode as u8,
            keyboard_mode: keyboard_mode as u8,
            confine_to,
            cursor: cursor as u32,
            button: button as u8,
            pad0: 0,
            modifiers: modifiers.iter().fold(0, |acc, modifier| acc | *modifier as u16),
        })?;

        self.replies.poll_error()
    }

    /// ungrab a button from the window,
    /// buttons are not valid modifiers
    pub fn ungrab_button(&mut self, button: Button, modifiers: Vec<KeyMask>) -> Result<(), Error> {
        self.sequence.skip();

        self.stream.send_encode(UngrabButton {
            opcode: Opcode::UNGRAB_BUTTON,
            button: button as u8,
            length: 3,
            grab_window: self.id(),
            modifiers: modifiers.iter().fold(0, |acc, modifier| acc | *modifier as u16),
            pad0: [0u8; 2],
        })?;

        self.replies.poll_error()
    }

    /// grab the pointer
    pub fn grab_pointer(
        &mut self,
        event_mask: Vec<EventMask>,
        cursor: Cursor,
        pointer_mode: PointerMode,
        keyboard_mode: KeyboardMode,
        owner_events: bool,
        confine_to: u32,
    ) -> Result<(), Error> {
        self.sequence.append(ReplyKind::GrabPointer)?;

        // TODO: un-hardcode time as current time

        self.stream.send_encode(GrabPointer {
            opcode: Opcode::GRAB_POINTER,
            owner_events: owner_events.then(|| 1).unwrap_or(0),
            length: 6,
            grab_window: self.id(),
            event_mask: event_mask.iter().fold(0, |acc, mask| acc | *mask as u16),
            pointer_mode: pointer_mode as u8,
            keyboard_mode: keyboard_mode as u8,
            confine_to,
            cursor: cursor as u32,
            time: 0,
        })?;

        match self.replies.wait()? {
            Reply::GrabPointer(_) => Ok(()),
            _ => unreachable!(),
        }
    }
}


