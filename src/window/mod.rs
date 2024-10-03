use crate::display::request::{self, *};
use crate::display::{Atom, Visual, Roots, Stream, TryClone};
use crate::display::xid;
use crate::keyboard::*;
use crate::proto::*;

use std::io::{Read, Write};


#[derive(Debug, Clone, Copy)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

impl From<u16> for WindowClass {
    fn from(value: u16) -> WindowClass {
        match value {
            0 => WindowClass::CopyFromParent,
            1 => WindowClass::InputOutput,
            2 => WindowClass::InputOnly,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PsuedoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl From<u8> for VisualClass {
    fn from(value: u8) -> VisualClass {
        match value {
            0 => VisualClass::StaticGray,
            1 => VisualClass::GrayScale,
            2 => VisualClass::StaticColor,
            3 => VisualClass::PsuedoColor,
            4 => VisualClass::TrueColor,
            5 => VisualClass::DirectColor,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BackingStore {
    NotUseful = 0,
    WhenMapped = 1,
    Always = 2,
}

#[derive(Clone, Copy)]
pub enum Gravity {
    Forget = 0,
    NorthWest = 1,
    North = 2,
    NorthEast = 3,
    West = 4,
    Center = 5,
    East = 6,
    SouthWest = 7,
    South = 8,
    SouthEast = 9,
    Static = 10,
}

#[derive(Clone, Copy)]
pub enum KeyMask {
    Shift = 0x0001,
    Lock = 0x0002,
    Control = 0x0004,
    Mod1 = 0x0008,
    Mod2 = 0x0010,
    Mod3 = 0x0020,
    Mod4 = 0x0040,
    Mod5 = 0x0080,
    Button1 = 0x0100,
    Button2 = 0x0200,
    Button3 = 0x0400,
    Button4 = 0x0800,
    Button5 = 0x1000,
}

#[derive(Clone, Copy)]
pub enum EventMask {
    NoEvent = 0,
    KeyPress = 1,
    KeyRelease = 2,
    ButtonPress = 4,
    ButtonRelease = 8,
    EnterWindow = 16,
    LeaveWindow = 32,
    PointerMotion = 64,
    PointerMotionHint = 128,
    Button1Motion = 256,
    Button2Motion = 512,
    Button3Motion = 1024,
    Button4Motion = 2048,
    Button5Motion = 4096,
    ButtonMotion = 8192,
    KeymapState = 16384,
    Exposure = 32768,
    VisibilityChange = 65536,
    StructureNotify = 131072,
    ResizeRedirect = 262144,
    SubstructureNotify = 524288,
    SubstructureRedirect = 1048576,
    FocusChange = 2097152,
    PropertyChange = 4194304,
    ColorMapChange = 8388608,
    OwnerGrabButton = 16777216,
}

// TODO: add cursors
#[derive(Clone)]
pub enum Cursor {
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
    pub fn mask(&self) -> u32 {
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
}

pub struct WindowValuesBuilder {
    values: Vec<WindowValue>,
    request: Vec<u8>,
    value_mask: u32,
}

impl WindowValuesBuilder {
    pub fn new(values: &[WindowValue]) -> WindowValuesBuilder {
        WindowValuesBuilder {
            values: values.to_vec(),
            request: Vec::new(),
            value_mask: 0,
        }
    }

    fn len(&self) -> u16 { self.values.len() as u16 }

    fn mask(&self, masks: Vec<EventMask>) -> u32 {
        masks.iter()
            .map(|event_mask| *event_mask as u32)
            .fold(0, |acc, x| acc | x)
    }

    fn insert_value(&mut self, value: WindowValue) {
        self.value_mask |= value.mask();

        match value {
            WindowValue::BgPixmap(value)
                | WindowValue::BgPixel(value)
                | WindowValue::BorderPixmap(value)
                | WindowValue::BorderPixel(value)
                | WindowValue::BackingPlane(value)
                | WindowValue::BackingPixel(value)
                | WindowValue::Colormap(value) => self.request.extend(request::encode(&value)),

            WindowValue::BitGravity(gravity)
                | WindowValue::WinGravity(gravity) => self.request.extend(request::encode(&(gravity as u32))),

            WindowValue::OverrideRedirect(value)
                | WindowValue::SaveUnder(value) => self.request.extend(request::encode(&(value as u8))),

            WindowValue::EventMask(masks)
                | WindowValue::DoNotPropogateMask(masks)=> self.request.extend(request::encode(&self.mask(masks))),

            WindowValue::Cursor(cursor) => self.request.extend(request::encode(&(cursor as u32))),
            WindowValue::BackingStore(store) => self.request.extend(request::encode(&(store as u32))),
        }
    }

    pub fn build(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.values.sort_by(|a, b| a.mask().partial_cmp(&b.mask()).unwrap());

        for value in self.values.clone() {
            self.insert_value(value);
        }

        Ok(self.request.clone())
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
    pub values: WindowValuesBuilder,
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
    ) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let screen = roots.first()?;

        stream.send_encode(GetWindowAttributes {
            opcode: Opcode::GET_WINDOW_ATTRIBUTES,
            pad0: 0,
            length: 2,
            wid: id,
        })?;

        sequence.append(ReplyKind::GetWindowAttributes)?;

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

    fn generic_window(&mut self, opcode: u8, length: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.send_encode(GenericWindow {
            opcode,
            pad0: 0,
            length,
            wid: self.id(),
        })?;

        self.sequence.skip();

        Ok(())
    }

    /// window id
    pub fn id(&self) -> u32 { self.id }

    /// window depth
    pub fn depth(&self) -> u8 { self.depth }

    /// visual assigned to the window
    pub fn visual(&self) -> Visual { self.visual.clone() }

    /// create a child window with provided window arguments
    pub fn create_window(&mut self, mut window: WindowArguments) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let window_values_request = window.values.build()?;
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
            value_mask: window.values.value_mask,
        })?;

        self.stream.send(&window_values_request)?;

        self.sequence.skip();

        Ok(Window::new(self.stream.try_clone()?, self.replies.clone(), self.sequence.clone(), window.visual, window.depth, wid))
    }

    /// change the attributes of a window
    pub fn change_attributes(&mut self, mut values: WindowValuesBuilder) -> Result<(), Box<dyn std::error::Error>> {
        let request = values.build()?;

        self.stream.send_encode(ChangeWindowAttributes {
            opcode: Opcode::CHANGE_WINDOW_ATTRIBUTES,
            pad0: 0,
            length: values.len() + 3,
            wid: self.id(),
            mask: values.value_mask,
        })?;

        self.stream.send(&request)?;

        self.sequence.skip();

        Ok(())
    }

    /// choose the events you want to recieve
    pub fn select_input(&mut self, events: &[EventMask]) -> Result<(), Box<dyn std::error::Error>> {
        self.change_attributes(WindowValuesBuilder::new(&[WindowValue::EventMask(events.to_vec())]))
    }

    /// become the child of a parent window
    pub fn reparent(&mut self, parent: Window<T>, x: u16, y: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.sequence.skip();

        self.stream.send_encode(ReparentWindow {
            opcode: Opcode::REPARENT_WINDOW,
            pad0: 0,
            length: 4,
            window: self.id(),
            parent: parent.id(),
            x,
            y,
        })
    }

    /// destroy the current window object
    pub fn destroy(mut self, kind: WindowKind) -> Result<(), Box<dyn std::error::Error>> {
        self.generic_window(kind.encode(Opcode::DESTROY_SUBWINDOWS, Opcode::DESTROY_WINDOW), 2)
    }

    /// map the window onto the screen
    pub fn map(&mut self, kind: WindowKind) -> Result<(), Box<dyn std::error::Error>> {
        self.generic_window(kind.encode(Opcode::MAP_SUBWINDOWS, Opcode::MAP_WINDOW), 2)
    }

    /// unmap the window
    pub fn unmap(&mut self, kind: WindowKind) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        self.sequence.skip();

        Ok(())
    }

    /// delete a property from a window
    pub fn delete_property(&mut self, property: Atom) -> Result<(), Box<dyn std::error::Error>> {
        self.generic_window(Opcode::DELETE_PROPERTY, 3)?;

        self.stream.send_encode(property.id())?;

        self.sequence.skip();

        Ok(())
    }

    /// get the value of a property from a window
    pub fn get_property(&mut self, property: Atom, type_: Atom, delete: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

        self.sequence.append(ReplyKind::GetProperty)?;

        match self.replies.wait()? {
            Reply::GetProperty { value } => Ok(value),
            _ => unreachable!(),
        }
    }

    /// get info about the pointer such as position
    pub fn query_pointer(&mut self) -> Result<QueryPointerResponse, Box<dyn std::error::Error>> {
        self.stream.send_encode(QueryPointer {
            opcode: Opcode::QUERY_POINTER,
            pad0: 0,
            length: 2,
            wid: self.id(),
        })?;

        self.sequence.append(ReplyKind::QueryPointer)?;

        match self.replies.wait()? {
            Reply::QueryPointer(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    /// grab some keys from the window,
    /// buttons are not valid modifiers
    pub fn grab_key(&mut self, modifier: Vec<KeyMask>, keycode: Keycode) {
    }
}


