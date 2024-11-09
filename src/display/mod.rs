//! Basic Display functionality for x11
//!
//!

pub(crate) mod auth;
pub(crate) mod error;
pub(crate) mod parse;
pub mod request;
pub(crate) mod xid;

use crate::extension::Extension;

#[cfg(feature = "xinerama")]
use crate::extension::xinerama::Xinerama;

#[cfg(feature = "ewmh")]
use crate::ewmh::EwmhAtoms;

use crate::keyboard::*;
use crate::proto::*;
use crate::window::*;

use error::Error;
use parse::Protocol;
use request::*;

use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;

// https://www.x.org/docs/XProtocol/proto.pdf

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Error::FailedToLock)
    };
}

const X_TCP_PORT: u16 = 6000;
const X_PROTOCOL: u16 = 11;
const X_PROTOCOL_REVISION: u16 = 0;

pub trait Streamable: Send + Sync + Read + Write {}

impl Streamable for File {}
impl Streamable for TcpStream {}
impl Streamable for UnixStream {}

pub struct Stream {
    reader: Arc<Mutex<dyn Streamable>>,
    writer: Arc<Mutex<dyn Streamable>>,
}

impl Clone for Stream {
    fn clone(&self) -> Stream {
        Stream {
            reader: self.reader.clone(),
            writer: self.writer.clone(),
        }
    }
}

impl Stream {
    pub fn new(reader: Arc<Mutex<dyn Streamable>>, writer: Arc<Mutex<dyn Streamable>>) -> Stream {
        Stream { reader, writer }
    }

    pub fn send(&self, request: &[u8]) -> Result<(), Error> {
        let mut lock = lock!(self.writer)?;

        lock.write_all(request)
            .map_err(|err| Error::Other { error: err.into() })
    }

    pub fn send_arr(&self, requests: &[Vec<u8>]) -> Result<(), Error> {
        let mut lock = lock!(self.writer)?;

        for request in requests {
            lock.write_all(request)
                .map_err(|err| Error::Other { error: err.into() })?;
        }

        Ok(())
    }

    pub fn send_pad(&self, request: &[u8]) -> Result<(), Error> {
        let mut lock = lock!(self.writer)?;

        lock.write_all(request)
            .map_err(|err| Error::Other { error: err.into() })?;

        lock.write_all(&vec![0u8; request::pad(request.len())])
            .map_err(|err| Error::Other { error: err.into() })?;

        Ok(())
    }

    pub fn send_encode<E>(&self, object: E) -> Result<(), Error> {
        self.send(request::encode(&object))
    }

    pub fn recv(&self, size: usize) -> Result<Vec<u8>, Error> {
        let mut lock = lock!(self.reader)?;
        let mut buffer = vec![0u8; size];

        match lock.read_exact(&mut buffer) {
            Ok(()) => Ok(buffer),
            Err(err) => Err(Error::Other { error: err.into() }),
        }
    }

    pub fn recv_str(&self, size: usize) -> Result<String, Error> {
        let bytes = self.recv(size)?;

        self.recv(size % 4)?;

        String::from_utf8(bytes).map_err(|_| Error::Utf8)
    }

    pub fn recv_decode<R>(&self) -> Result<R, Error> {
        let bytes = self.recv(std::mem::size_of::<R>())?;

        Ok(request::decode(&bytes))
    }
}

/// an atom in the x11 protocol is an integer representing a string
/// atoms in the range 1..=68 are predefined (only 1..=20 implemented so far)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom {
    id: u32,
}

impl Atom {
    /// this is not really a atom, its just for functions that can take AnyPropertyValue
    pub const ANY_PROPERTY_TYPE: Atom = Atom::new(0);

    pub const PRIMARY: Atom = Atom::new(1);
    pub const SECONDARY: Atom = Atom::new(2);
    pub const ARC: Atom = Atom::new(3);
    pub const ATOM: Atom = Atom::new(4);
    pub const BITMAP: Atom = Atom::new(5);
    pub const CARDINAL: Atom = Atom::new(6);
    pub const COLORMAP: Atom = Atom::new(7);
    pub const CURSOR: Atom = Atom::new(8);
    pub const CUT_BUFFER0: Atom = Atom::new(9);
    pub const CUT_BUFFER1: Atom = Atom::new(10);
    pub const CUT_BUFFER2: Atom = Atom::new(11);
    pub const CUT_BUFFER3: Atom = Atom::new(12);
    pub const CUT_BUFFER4: Atom = Atom::new(13);
    pub const CUT_BUFFER5: Atom = Atom::new(14);
    pub const CUT_BUFFER6: Atom = Atom::new(15);
    pub const CUT_BUFFER7: Atom = Atom::new(16);
    pub const DRAWABLE: Atom = Atom::new(17);
    pub const FONT: Atom = Atom::new(18);
    pub const INTEGER: Atom = Atom::new(19);
    pub const PIXMAP: Atom = Atom::new(20);
    pub const WINDOW: Atom = Atom::new(33);

    /// create a new atom from its id
    pub const fn new(id: u32) -> Atom {
        Atom { id }
    }

    /// get the id of the atom
    pub fn id(&self) -> u32 {
        self.id
    }

    /// returns true of the atom is null
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

#[derive(Debug, Clone)]
pub struct Visual {
    pub id: u32,
    pub class: VisualClass,
}

impl Visual {
    pub fn new(response: VisualResponse) -> Visual {
        Visual {
            id: response.visual_id,
            class: VisualClass::from(response.class),
        }
    }
}

#[derive(Clone)]
pub struct Depth {
    depth: u8,
    length: u16,
    visuals: Vec<Visual>,
}

impl Depth {
    pub fn new(response: DepthResponse) -> Depth {
        Depth {
            depth: response.depth,
            length: response.visuals_len,
            visuals: Vec::new(),
        }
    }

    pub fn extend(&mut self, responses: &[VisualResponse]) {
        self.visuals
            .extend(responses.iter().map(|response| Visual::new(*response)));
    }
}

#[derive(Clone, Copy)]
pub struct KeycodeRange {
    pub min: u8,
    pub max: u8,
}

impl KeycodeRange {
    pub fn new(min: u8, max: u8) -> KeycodeRange {
        KeycodeRange { min, max }
    }
}

#[derive(Clone)]
pub struct Screen {
    pub response: ScreenResponse,
    pub depths: Vec<Depth>,
}

impl Screen {
    pub fn new(response: ScreenResponse) -> Screen {
        Screen {
            response,
            depths: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Roots {
    roots: Vec<Screen>,
}

impl Roots {
    pub fn new() -> Roots {
        Roots { roots: Vec::new() }
    }

    pub fn first(&self) -> Result<&Screen, Error> {
        self.roots.first().ok_or(Error::NoScreens)
    }

    pub fn visual_from_id(&self, id: u32) -> Result<Visual, Error> {
        for screen in &self.roots {
            for depth in &screen.depths {
                match depth.visuals.iter().find(|visual| visual.id == id) {
                    Some(visual) => return Ok(visual.clone()),
                    None => {}
                }
            }
        }

        Err(Error::InvalidId)
    }

    pub fn push(&mut self, screen: Screen) {
        self.roots.push(screen);
    }
}

pub struct Display {
    pub(crate) stream: Stream,
    pub(crate) events: Queue<Event>,
    pub(crate) replies: Queue<Reply>,
    pub(crate) roots: Roots,
    pub(crate) setup: SuccessResponse,
    pub(crate) sequence: SequenceManager,
}

impl Clone for Display {
    /// get a thread safe clone of the display, this still points to the same event queue so
    /// listening for events in multiple threads is a bad and unreliable idea
    fn clone(&self) -> Display {
        Display {
            stream: self.stream.clone(),
            events: self.events.clone(),
            replies: self.replies.clone(),
            roots: self.roots.clone(),
            setup: self.setup.clone(),
            sequence: self.sequence.clone(),
        }
    }
}

impl Display {
    pub fn connect<'a>(stream: Stream) -> Result<Display, Error> {
        let errors: Arc<Mutex<Vec<Error>>> = Arc::new(Mutex::new(Vec::new()));

        let mut display = Display {
            stream,
            events: Queue::new(errors.clone()),
            replies: Queue::new(errors.clone()),
            roots: Roots::new(),
            setup: SuccessResponse::default(),
            sequence: SequenceManager::new(),
        };

        display.setup()?;

        Ok(display)
    }

    /// wait for the next event
    pub fn next_event(&mut self) -> Result<Event, Error> {
        self.events.wait()
    }

    /// returns true if an event is ready
    pub fn poll_event(&mut self) -> Result<bool, Error> {
        self.events.poll()
    }

    /// get the window from its id
    pub fn window_from_id(&self, id: u32) -> Result<Window, Error> {
        Window::from_id(
            self.stream.clone(),
            self.replies.clone(),
            self.sequence.clone(),
            self.roots.clone(),
            id,

            #[cfg(feature = "ewmh")]
            self.get_ewmh_atoms()?,
        )
    }

    /// get the default root window of a display
    pub fn default_root_window(&self) -> Result<Window, Error> {
        let screen = self.roots.first()?;

        Ok(Window::new(
            self.stream.clone(),
            self.replies.clone(),
            self.sequence.clone(),
            self.roots.visual_from_id(screen.response.root_visual)?,
            screen.response.root_depth,
            screen.response.root,

            #[cfg(feature = "ewmh")]
            self.get_ewmh_atoms()?,
        ))
    }

    /// query an extension and if its active get its major opcode
    pub fn query_extension(&self, extension: Extension) -> Result<QueryExtensionResponse, Error> {
        self.sequence.append(ReplyKind::QueryExtension)?;

        let request = QueryExtension {
            opcode: Opcode::QUERY_EXTENSION,
            pad0: 0,
            length: 2 + (extension.len() as u16 + request::pad(extension.len()) as u16) / 4,
            name_len: extension.len() as u16,
            pad1: 0,
        };

        let extension = extension.to_string();

        self.stream.send(
            &[
                request::encode(&request).to_vec(),
                extension.as_bytes().to_vec(),
                vec![0u8; request::pad(extension.as_bytes().len())],
            ]
            .concat(),
        )?;

        match self.replies.wait()? {
            Reply::QueryExtension(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    /// query for the xinerama extension and return a structure with its methods

    #[cfg(feature = "xinerama")]
    pub fn query_xinerama(&self) -> Result<Xinerama, Error> {
        let extension = self.query_extension(Extension::Xinerama)?;

        Ok(Xinerama::new(
            self.stream.clone(),
            self.replies.clone(),
            self.sequence.clone(),
            extension.major_opcode,
        ))
    }

    /// this request returns the current focused window
    pub fn get_input_focus(&self) -> Result<GetInputFocusResponse, Error> {
        self.sequence.append(ReplyKind::GetInputFocus)?;

        self.stream.send_encode(GetInputFocus {
            opcode: Opcode::GET_INPUT_FOCUS,
            pad0: 0,
            length: 1,
        })?;

        match self.replies.wait()? {
            Reply::GetInputFocus(response) => Ok(response),
            _ => unreachable!(),
        }
    }

    /// get an atom from its name
    pub fn intern_atom<'a>(&self, name: &'a str, only_if_exists: bool) -> Result<Atom, Error> {
        self.sequence.append(ReplyKind::InternAtom)?;

        let request = InternAtom {
            opcode: Opcode::INTERN_ATOM,
            only_if_exists: only_if_exists.then(|| 1).unwrap_or(0),
            length: 2 + (name.len() as u16 + request::pad(name.len()) as u16) / 4,
            name_len: name.len() as u16,
            pad1: [0u8; 2],
        };

        self.stream.send(
            &[
                request::encode(&request).to_vec(),
                name.as_bytes().to_vec(),
                vec![0u8; request::pad(name.as_bytes().len())],
            ]
            .concat(),
        )?;

        match self.replies.wait()? {
            Reply::InternAtom(response) => match response.atom {
                u32::MIN => Err(Error::InvalidAtom),
                _ => Ok(Atom::new(response.atom)),
            },
            _ => unreachable!(),
        }
    }

    /// get the owner of a selection, (this function returns the window id, use
    /// display::window_from_id to get the structure)
    pub fn get_selection_owner(&self, selection: Atom) -> Result<u32, Error> {
        self.sequence.append(ReplyKind::GetSelectionOwner)?;

        self.stream.send_encode(GetSelectionOwner {
            opcode: Opcode::GET_SELECTION_OWNER,
            pad0: 0,
            length: 2,
            selection: selection.id(),
        })?;

        match self.replies.wait()? {
            Reply::GetSelectionOwner(response) => Ok(response.owner),
            _ => unreachable!(),
        }
    }

    /// get the min and max keycode
    pub fn display_keycodes(&self) -> KeycodeRange {
        KeycodeRange::new(self.setup.min_keycode, self.setup.max_keycode)
    }

    /// get the keyboard mapping from the server
    pub fn get_keyboard_mapping(&self) -> Result<(Vec<Keysym>, u8), Error> {
        self.sequence.append(ReplyKind::GetKeyboardMapping)?;

        self.stream.send_encode(GetKeyboardMapping {
            opcode: Opcode::GET_KEYBOARD_MAPPING,
            pad0: 0,
            length: 2,
            first: self.setup.min_keycode,
            count: self.setup.max_keycode - self.setup.min_keycode + 1,
            pad1: [0u8; 2],
        })?;

        match self.replies.wait()? {
            Reply::GetKeyboardMapping {
                keysyms,
                keysyms_per_keycode,
            } => Ok((keysyms, keysyms_per_keycode)),
            _ => unreachable!(),
        }
    }

    /// get the keysym from a keycode
    pub fn keysym_from_keycode(&self, keycode: u8) -> Result<Keysym, Error> {
        let (keysyms, keysyms_per_keycode) = self.get_keyboard_mapping()?;

        Ok(keysyms[(keycode - self.setup.min_keycode) as usize * keysyms_per_keycode as usize])
    }

    /// get the keysym from a character
    pub fn keysym_from_character(&self, character: char) -> Result<Keysym, Error> {
        let (keysyms, _) = self.get_keyboard_mapping()?;

        keysyms
            .iter()
            .find(|keysym| keysym.character().map(|c| c == character).unwrap_or(false))
            .map(|keysym| *keysym)
            .ok_or(Error::InvalidKeysym)
    }

    /// get the keycode from a keysym
    pub fn keycode_from_keysym(&self, keysym: Keysym) -> Result<u8, Error> {
        let (keysyms, keysyms_per_keycode) = self.get_keyboard_mapping()?;

        keysyms
            .iter()
            .enumerate()
            .find(|(_, x)| **x == keysym)
            .map(|(index, _)| {
                ((index / keysyms_per_keycode as usize) + self.setup.min_keycode as usize) as u8
            })
            .ok_or(Error::InvalidKeysym)
    }

    /// ungrab the pointer
    pub fn ungrab_pointer(&self) -> Result<(), Error> {
        self.sequence.skip();

        // TODO: un-hardcode current time

        self.stream.send_encode(UngrabPointer {
            opcode: Opcode::UNGRAB_POINTER,
            pad0: 0,
            length: 2,
            time: 0,
        })?;

        self.replies.poll_error()
    }

    fn endian(&self) -> u8 {
        cfg!(target_endian = "little")
            .then(|| 0x6c)
            .unwrap_or_else(|| 0x42)
    }

    fn read_setup(&mut self) -> Result<(), Error> {
        self.setup = self.stream.recv_decode()?;

        let _vendor = self.stream.recv_str(self.setup.vendor_len as usize)?;

        let bytes = self
            .stream
            .recv(std::mem::size_of::<PixmapFormat>() * self.setup.pixmap_formats_len as usize)?;

        let _formats: &[PixmapFormat] =
            request::decode_slice(&bytes, self.setup.pixmap_formats_len as usize);

        for _ in 0..self.setup.roots_len {
            let mut screen = Screen::new(self.stream.recv_decode()?);

            for _ in 0..screen.response.allowed_depths_len {
                let mut depth = Depth::new(self.stream.recv_decode()?);

                let bytes = self
                    .stream
                    .recv(std::mem::size_of::<VisualResponse>() * depth.length as usize)?;

                depth.extend(request::decode_slice(&bytes, depth.length as usize));

                screen.depths.push(depth);
            }

            self.roots.push(screen);
        }

        let stream = self.stream.clone();
        let events = self.events.clone();
        let replies = self.replies.clone();
        let sequence = self.sequence.clone();
        let roots = self.roots.clone();

        thread::spawn(move || {
            let mut listener = EventListener::new(stream, events, replies.clone(), sequence, roots);

            if let Err(err) = listener.listen() {
                let _ = replies.push_error(err);
            }
        });

        xid::setup(self.setup.resource_id_base, self.setup.resource_id_mask)?;

        Ok(())
    }

    fn setup<'a>(&mut self) -> Result<(), Error> {
        let entry = auth::entry()?;

        let request = SetupRequest::new(
            self.endian(),
            X_PROTOCOL,
            X_PROTOCOL_REVISION,
            entry.name.len() as u16,
            entry.data.len() as u16,
        );

        self.stream.send(request::encode(&request))?;

        self.stream.send_arr(&[
            entry.name.clone(),
            vec![0u8; request::pad(entry.name.len())],
            entry.data.clone(),
            vec![0u8; request::pad(entry.data.len())],
        ])?;

        let response: SetupResponse = self.stream.recv_decode()?;

        match response.status {
            1 => self.read_setup(),
            0 => Err(Error::SetupFailed {
                reason: self.stream.recv_str(response.padding as usize)?,
            }),
            2 => Err(Error::Authenthicate),
            _ => Err(Error::InvalidStatus),
        }
    }
}

pub struct EventListener {
    stream: Stream,
    events: Queue<Event>,
    replies: Queue<Reply>,
    sequence: SequenceManager,
    roots: Roots,
}

impl EventListener {
    pub fn new(
        stream: Stream,
        events: Queue<Event>,
        replies: Queue<Reply>,
        sequence: SequenceManager,
        roots: Roots,
    ) -> EventListener {
        EventListener {
            stream,
            events,
            replies,
            sequence,
            roots,
        }
    }

    fn handle_reply(&mut self, event: GenericEvent) -> Result<(), Error> {
        let sequence = self.sequence.get(event.sequence)?;

        match sequence.kind {
            ReplyKind::InternAtom => {
                let response: InternAtomResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::InternAtom(response))?;
            }
            ReplyKind::GetWindowAttributes => {
                let response: GetWindowAttributesResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetWindowAttributes(response))?;
            }
            ReplyKind::GetGeometry => {
                let response: GetGeometryResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetGeometry(response))?;
            }
            ReplyKind::GrabPointer => {
                let response: GrabPointerResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GrabPointer(response))?;
            }
            ReplyKind::QueryPointer => {
                let response: QueryPointerResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::QueryPointer(response))?;
            }
            ReplyKind::QueryExtension => {
                let response: QueryExtensionResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::QueryExtension(response))?;
            }
            ReplyKind::GetSelectionOwner => {
                let response: GetSelectionOwnerResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetSelectionOwner(response))?;
            }
            #[cfg(feature = "xinerama")]
            ReplyKind::XineramaIsActive => {
                let response: XineramaIsActiveResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::XineramaIsActive(response))?;
            }
            #[cfg(feature = "xinerama")]
            ReplyKind::XineramaQueryScreens => {
                let response: XineramaQueryScreensResponse = self.stream.recv_decode()?;

                let mut screens: Vec<XineramaScreenInfo> = Vec::new();

                for _ in 0..response.number {
                    screens.push(self.stream.recv_decode()?);
                }

                self.replies.push(Reply::XineramaQueryScreens { screens })?;
            }
            ReplyKind::GetInputFocus => {
                let response: GetInputFocusResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetInputFocus(response))?;
            }
            ReplyKind::GetProperty => {
                let response: GetPropertyResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetProperty {
                    type_: Atom::new(response.type_),
                    value: self.stream.recv(response.value_len as usize)?,
                })?;

                self.stream
                    .recv(request::pad(response.value_len as usize))?;
            }
            ReplyKind::GetKeyboardMapping => {
                let response: KeyboardMappingResponse = self.stream.recv_decode()?;

                let bytes = self.stream.recv(4 * response.length as usize)?;

                let keysyms = request::decode_slice::<u32>(&bytes, response.length as usize);

                self.replies.push(Reply::GetKeyboardMapping {
                    keysyms: keysyms
                        .iter()
                        .map(|value| Keysym::new(*value))
                        .collect::<Vec<Keysym>>(),
                    keysyms_per_keycode: event.detail,
                })?;
            }
        }

        Ok(())
    }

    // TODO: there is a lot of repetition here, it may be possible to procedurally generate this
    // through macros instead

    fn handle_event(&mut self, generic: GenericEvent) -> Result<(), Error> {
        match generic.opcode & 0b0111111 {
            Response::ERROR => {
                let error: ErrorEvent = self.stream.recv_decode()?;

                self.replies.push_error(Error::Event {
                    error: ErrorCode::from(generic.detail),
                    major_opcode: error.major_opcode,
                    minor_opcode: error.minor_opcode,
                    bad_value: error.bad_value,
                    sequence: generic.sequence,
                })
            }
            Response::REPLY => {
                self.handle_reply(generic)?;

                Ok(())
            }
            Response::KEY_PRESS | Response::KEY_RELEASE => {
                let key_event: KeyEvent = self.stream.recv_decode()?;

                self.events.push(Event::KeyEvent {
                    kind: match generic.opcode & 0b0111111 {
                        Response::KEY_PRESS => EventKind::Press,
                        Response::KEY_RELEASE => EventKind::Release,
                        _ => unreachable!(),
                    },
                    coordinates: Coordinates::new(
                        key_event.event_x,
                        key_event.event_y,
                        key_event.root_x,
                        key_event.root_y,
                    ),
                    window: key_event.event,
                    root: key_event.root,
                    subwindow: key_event.child,
                    state: key_event.state,
                    keycode: generic.detail,
                    send_event: key_event.same_screen == 0,
                })
            }
            Response::BUTTON_PRESS | Response::BUTTON_RELEASE => {
                let button_event: ButtonEvent = self.stream.recv_decode()?;

                self.events.push(Event::ButtonEvent {
                    kind: match generic.opcode & 0b0111111 {
                        Response::BUTTON_PRESS => EventKind::Press,
                        Response::BUTTON_RELEASE => EventKind::Release,
                        _ => unreachable!(),
                    },
                    coordinates: Coordinates::new(
                        button_event.event_x,
                        button_event.event_y,
                        button_event.root_x,
                        button_event.root_y,
                    ),
                    window: button_event.event,
                    root: button_event.root,
                    subwindow: button_event.child,
                    state: button_event.state,
                    button: Button::from(generic.detail),
                    send_event: button_event.same_screen == 0,
                })
            }
            Response::MOTION_NOTIFY => {
                let motion_notify: MotionNotify = self.stream.recv_decode()?;

                self.events.push(Event::MotionNotify {
                    coordinates: Coordinates::new(
                        motion_notify.event_x,
                        motion_notify.event_y,
                        motion_notify.root_x,
                        motion_notify.root_y,
                    ),
                    window: motion_notify.event,
                    root: motion_notify.root,
                    subwindow: motion_notify.child,
                    state: motion_notify.state,
                    send_event: motion_notify.same_screen == 0,
                })
            }
            Response::ENTER_NOTIFY => {
                let event: EnterNotify = self.stream.recv_decode()?;

                self.events.push(Event::EnterNotify {
                    root: event.root,
                    window: event.event,
                    child: event.child,
                    coordinates: Coordinates::new(
                        event.event_x,
                        event.event_y,
                        event.root_x,
                        event.root_y,
                    ),
                    state: event.state,
                    mode: EnterMode::from(event.mode),
                    focus: (event.sf & 0x01) != 0,
                    same_screen: (event.sf & 0x02) != 0,
                })
            }
            Response::FOCUS_IN => {
                let event: FocusIn = self.stream.recv_decode()?;

                self.events.push(Event::FocusIn {
                    detail: FocusDetail::from(generic.detail),
                    mode: FocusMode::from(event.mode),
                    window: event.event,
                })
            }
            Response::FOCUS_OUT => {
                let event: FocusOut = self.stream.recv_decode()?;

                self.events.push(Event::FocusOut {
                    detail: FocusDetail::from(generic.detail),
                    mode: FocusMode::from(event.mode),
                    window: event.event,
                })
            }
            Response::CREATE_NOTIFY => {
                let event: CreateNotify = self.stream.recv_decode()?;

                self.events.push(Event::CreateNotify {
                    parent: event.event,
                    window: event.window,
                    x: event.x,
                    y: event.y,
                    width: event.height,
                    height: event.height,
                })
            }
            Response::DESTROY_NOTIFY => {
                let event: DestroyNotify = self.stream.recv_decode()?;

                self.events.push(Event::DestroyNotify {
                    event: event.event,
                    window: event.window,
                })
            }
            Response::UNMAP_NOTIFY => {
                let event: UnmapNotify = self.stream.recv_decode()?;

                self.events.push(Event::UnmapNotify {
                    event: event.event,
                    window: event.window,
                    configure: event.from_configure == 0,
                })
            }
            Response::MAP_NOTIFY => {
                let event: MapNotify = self.stream.recv_decode()?;

                self.events.push(Event::MapNotify {
                    event: event.event,
                    window: event.window,
                    override_redirect: event.override_redirect == 0,
                })
            }
            Response::MAP_REQUEST => {
                let event: MapReq = self.stream.recv_decode()?;

                self.events.push(Event::MapRequest {
                    parent: event.parent,
                    window: event.window,
                })
            }
            Response::REPARENT_NOTIFY => {
                let event: ReparentNotify = self.stream.recv_decode()?;

                self.events.push(Event::ReparentNotify {
                    event: event.event,
                    parent: event.parent,
                    window: event.window,
                    x: event.x,
                    y: event.y,
                    override_redirect: event.override_redirect == 0,
                })
            }
            Response::CONFIGURE_NOTIFY => {
                let event: ConfigNotify = self.stream.recv_decode()?;

                self.events.push(Event::ConfigureNotify {
                    event: event.event,
                    window: event.window,
                    above_sibling: event.above_sibling,
                    x: event.x,
                    y: event.y,
                    width: event.height,
                    height: event.height,
                    border_width: event.border_width,
                    override_redirect: event.override_redirect == 0,
                })
            }
            Response::CONFIGURE_REQUEST => {
                let event: ConfigReq = self.stream.recv_decode()?;

                let mut values: Vec<ConfigureValue> = Vec::new();

                for bit in 0..16 {
                    if ((event.value_mask & (0x1 << bit)) >> bit) == 0x1 {
                        match 1 << bit {
                            0x1 => values.push(ConfigureValue::X(event.x)),
                            0x2 => values.push(ConfigureValue::Y(event.y)),
                            0x4 => values.push(ConfigureValue::Width(event.width)),
                            0x8 => values.push(ConfigureValue::Height(event.height)),
                            0x10 => values.push(ConfigureValue::Border(event.border_width)),
                            0x20 => values.push(ConfigureValue::Sibling(event.sibling)),
                            _ => {}
                        }
                    }
                }

                self.events.push(Event::ConfigureRequest {
                    window: event.window,
                    values,
                })
            }
            Response::GRAVITY_NOTIFY => {
                let event: GravityNotify = self.stream.recv_decode()?;

                self.events.push(Event::GravityNotify {
                    event: event.event,
                    window: event.window,
                    x: event.x,
                    y: event.y,
                })
            }
            Response::CIRCULATE_NOTIFY => {
                let event: CircNotify = self.stream.recv_decode()?;

                self.events.push(Event::CirculateNotify {
                    event: event.event,
                    window: event.window,
                    place: Place::from(event.place),
                })
            }
            Response::CIRCULATE_REQUEST => {
                let event: CircReq = self.stream.recv_decode()?;

                self.events.push(Event::CirculateRequest {
                    parent: event.event,
                    window: event.window,
                    place: Place::from(event.place),
                })
            }
            Response::PROPERTY_NOTIFY => {
                let event: PropertyNotify = self.stream.recv_decode()?;

                self.events.push(Event::PropertyNotify {
                    window: event.window,
                    atom: Atom::new(event.atom),
                    time: event.time,
                    state: PropertyState::from(event.state),
                })
            }
            Response::SELECTION_CLEAR => {
                let event: SelectionClear = self.stream.recv_decode()?;

                self.events.push(Event::SelectionClear {
                    time: event.time,
                    owner: event.owner,
                    selection: Atom::new(event.selection),
                })
            }
            Response::SELECTION_REQUEST => {
                let event: SelectionReq = self.stream.recv_decode()?;

                self.events.push(Event::SelectionRequest {
                    time: event.time,
                    owner: event.requestor,
                    selection: Atom::new(event.selection),
                    target: Atom::new(event.target),
                    property: Atom::new(event.property),
                })
            }
            Response::SELECTION_NOTIFY => {
                let event: SelectionNotify = self.stream.recv_decode()?;

                self.events.push(Event::SelectionNotify {
                    time: event.time,
                    requestor: event.requestor,
                    selection: Atom::new(event.selection),
                    target: Atom::new(event.target),
                    property: Atom::new(event.property),
                })?;

                Ok(())
            }
            Response::CLIENT_MESSAGE => {
                let event: ClientMessage = self.stream.recv_decode()?;
                let data: [u8; 20] = self.stream.recv_decode()?;

                self.events.push(Event::ClientMessage {
                    format: generic.detail,
                    window: event.window,
                    type_: Atom::new(event.type_),
                    data: ClientMessageData::Byte(data),
                })
            }
            Response::MAPPING_NOTIFY => {
                let event: MappingNotify = self.stream.recv_decode()?;

                self.events.push(Event::MappingNotify {
                    request: event.request,
                    keycode: event.keycode,
                    count: event.count,
                })
            }
            _ => Ok(()),
        }
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        loop {
            let event: GenericEvent = self.stream.recv_decode()?;

            self.handle_event(event)?;
        }
    }
}

fn open_tcp<'a>(host: SocketAddr) -> Result<Display, Error> {
    let tcp_stream = TcpStream::connect(host).map_err(|_| Error::Stream)?;

    tcp_stream
        .set_nonblocking(false)
        .map_err(|_| Error::Stream)?;

    Display::connect(Stream::new(
        Arc::new(Mutex::new(
            tcp_stream.try_clone().map_err(|_| Error::Stream)?,
        )),
        Arc::new(Mutex::new(tcp_stream)),
    ))
}

fn open_unix<'a>(path: String) -> Result<Display, Error> {
    let unix_stream = UnixStream::connect(path).map_err(|_| Error::Stream)?;

    unix_stream
        .set_nonblocking(false)
        .map_err(|_| Error::Stream)?;

    Display::connect(Stream::new(
        Arc::new(Mutex::new(
            unix_stream.try_clone().map_err(|_| Error::Stream)?,
        )),
        Arc::new(Mutex::new(unix_stream)),
    ))
}

/// Open a connection to the x11 server that controls a display.
///
/// This function is essentialy the yaxi equivalent to [XOpenDisplay](https://tronche.com/gui/x/xlib/display/opening.html).
///
/// On POSIX-conformant systems, the display name or DISPLAY environment variable can be a string in the format:
/// `hostname:number.screen_number`
///
/// - `hostname` - Specifies the name of the host machine on which the display is physically attached. You follow the hostname with either a single colon (:) or a double colon (::).
/// - `number` - Specifies the number of the display server on that host machine. You may optionally follow this display number with a period (.). A single CPU can have more than one display. Multiple displays are usually numbered starting with zero.
/// - `screen_number` - Specifies the screen to be used on that server. Multiple screens can be controlled by a single X server. The screen_number sets an internal variable that can be accessed by using the DefaultScreen() macro or the XDefaultScreen() function if you are using languages other than C (see "Display Macros").
///
/// # Example: connect using $DISPLAY
///
/// ```no_run
/// use yaxi::display;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut _display = display::open(None)?;
///
///     Ok(())
/// }
/// ```
///
/// # Example: connect using custom display string
///
/// ```no_run
/// use yaxi::display;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut _display = display::open(Some("13.37.13.37/tcp:69.420"))?;
///
///     Ok(())
/// }
/// ```
pub fn open(display: Option<&str>) -> Result<Display, Error> {
    let info = parse::parse(display)?;

    match (info.protocol, info.host.is_empty()) {
        (Protocol::TcpSocket, true) => open_tcp(SocketAddr::from((
            [127, 0, 0, 1],
            X_TCP_PORT + info.display,
        ))),
        (Protocol::UnixSocket, true) => open_unix(format!("/tmp/.X11-unix/X{}", info.display)),
        (Protocol::TcpSocket, false) => open_tcp(SocketAddr::from((
            IpAddr::from_str(&info.host).map_err(|_| Error::InvalidDisplay)?,
            X_TCP_PORT + info.display,
        ))),
        (Protocol::UnixSocket, false) => open_unix(info.host),
    }
}
