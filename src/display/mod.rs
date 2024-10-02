pub(crate) mod error;
pub(crate) mod request;
pub(crate) mod auth;
pub(crate) mod xid;

use crate::proto::*;
use crate::window::*;
use request::*;
use error::Error;

use std::os::unix::net::UnixStream;
use std::net::{SocketAddr, TcpStream};
use std::io::{Read, Write};
use std::fs::File;
use std::thread;

// https://www.x.org/docs/XProtocol/proto.pdf


const X_TCP_PORT: u16 = 6000;
const X_PROTOCOL: u16 = 11;
const X_PROTOCOL_REVISION: u16 = 0;

pub trait TryClone {
    fn try_clone(&self) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}

impl TryClone for File {
    fn try_clone(&self) -> Result<Box<File>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
}

impl TryClone for TcpStream {
    fn try_clone(&self) -> Result<Box<TcpStream>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
}

impl TryClone for UnixStream {
    fn try_clone(&self) -> Result<Box<UnixStream>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
}

pub struct Stream<T> {
    inner: Box<T>,
}

impl<T> Clone for Stream<T> where T: Send + Sync + Read + Write + TryClone {
    fn clone(&self) -> Stream<T> {
        self.try_clone().expect("failed to clone")
    }
}

impl<T> Stream<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(inner: T) -> Stream<T> {
        Stream {
            inner: Box::new(inner),
        }
    }

    pub fn try_clone(&self) -> Result<Stream<T>, Box<dyn std::error::Error>> {
        Ok(Stream {
            inner: self.inner.try_clone()?,
        })
    }

    pub fn send(&mut self, request: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.write_all(request).map_err(|err| err.into())
    }

    pub fn send_arr(&mut self, requests: &[Vec<u8>]) -> Result<(), Box<dyn std::error::Error>> {
        for request in requests {
            self.send(request)?;
        }

        Ok(())
    }

    pub fn send_pad(&mut self, request: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.send(request)?;

        self.send(&vec![0u8; request::pad(request.len())])?;

        Ok(())
    }

    pub fn send_encode<E>(&mut self, object: E) -> Result<(), Box<dyn std::error::Error>> {
        self.send(request::encode(&object))
    }

    pub fn recv(&mut self, size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; size];

        match self.inner.read_exact(&mut buffer) {
            Ok(()) => Ok(buffer),
            Err(err) => Err(err.into()),
        }
    }

    pub fn recv_str(&mut self, size: usize) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.recv(size)?;

        self.recv(size % 4)?;

        Ok(String::from_utf8(bytes)?)
    }

    pub fn recv_decode<R>(&mut self) -> Result<R, Box<dyn std::error::Error>> {
        let bytes = self.recv(std::mem::size_of::<R>())?;

        Ok(request::decode(&bytes))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Atom {
    id: u32,
}

impl Atom {
    pub const CARDINAL: Atom = Atom::new(6);

    pub const fn new(id: u32) -> Atom {
        Atom {
            id,
        }
    }

    pub fn id(&self) -> u32 { self.id }
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
        self.visuals.extend(responses.iter().map(|response| Visual::new(*response)));
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
        Roots {
            roots: Vec::new(),
        }
    }

    pub fn first(&self) -> Result<&Screen, Box<dyn std::error::Error>> {
        self.roots.first().ok_or(Box::new(Error::NoScreens))
    }

    pub fn visual_from_id(&self, id: u32) -> Result<Visual, Box<dyn std::error::Error>> {
        for screen in &self.roots {
            for depth in &screen.depths {
                match depth.visuals.iter().find(|visual| visual.id == id) {
                    Some(visual) => return Ok(visual.clone()),
                    None => {},
                }
            }
        }

        Err(Box::new(Error::InvalidId))
    }

    pub fn push(&mut self, screen: Screen) {
        self.roots.push(screen);
    }
}

pub struct Display<T> where T: Send + Sync + Read + Write + TryClone {
    stream: Stream<T>,
    events: Queue<Event>,
    replies: Queue<Reply>,
    roots: Roots,
    sequence: SequenceManager,
}

impl<T> Display<T> where T: Send + Sync + Read + Write + TryClone + 'static {
    pub fn connect<'a>(inner: T) -> Result<Display<T>, Box<dyn std::error::Error>> {
        let mut display = Display {
            stream: Stream::new(inner),
            events: Queue::new(),
            replies: Queue::new(),
            roots: Roots::new(),
            sequence: SequenceManager::new(),
        };

        display.setup()?;

        Ok(display)
    }

    /// wait for the next event
    pub fn next_event(&mut self) -> Result<Event, Box<dyn std::error::Error>> {
        self.events.wait()
    }

    /// returns true if an event is ready
    pub fn poll_event(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        self.events.poll()
    }

    /// get the window from its id
    pub fn window_from_id(&self, id: u32) -> Result<Window<T>, Box<dyn std::error::Error>> {
        Window::from_id(self.stream.clone(), self.replies.clone(), self.sequence.clone(), self.roots.clone(), id)
    }

    /// get the default root window of a display
    pub fn default_root_window(&self) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let stream = self.stream.try_clone()?;
        let screen = self.roots.first()?;

        Ok(Window::<T>::new(stream, self.replies.clone(), self.sequence.clone(), self.roots.visual_from_id(screen.response.root_visual)?, screen.response.root_depth, screen.response.root))
    }

    /// get an atom from its name
    pub fn intern_atom<'a>(&mut self, name: &'a str, only_if_exists: bool) -> Result<Atom, Box<dyn std::error::Error>> {
        let request = InternAtom {
            opcode: Opcode::INTERN_ATOM,
            only_if_exists: only_if_exists.then(|| 1).unwrap_or(0),
            length: 2 + (name.len() as u16 + request::pad(name.len()) as u16) / 4,
            name_len: name.len() as u16,
            pad1: [0u8; 2],
        };

        self.stream.send(request::encode(&request))?;

        self.stream.send_pad(name.as_bytes())?;

        self.sequence.append(ReplyKind::InternAtom)?;

        match self.replies.wait()? {
            Reply::InternAtom(response) => match response.atom {
                u32::MIN => Err(Box::new(Error::InvalidAtom)),
                _ => Ok(Atom::new(response.atom)),
            },
            _ => unreachable!(),
        }
    }

    fn endian(&self) -> u8 {
        cfg!(target_endian = "little")
            .then(|| 0x6c)
            .unwrap_or_else(|| 0x42)
    }

    fn read_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response: SuccessResponse = self.stream.recv_decode()?;

        println!("response: {:?}", response);

        let vendor = self.stream.recv_str(response.vendor_len as usize)?;

        println!("vendor: {}", vendor);

        let bytes = self.stream.recv(std::mem::size_of::<PixmapFormat>() * response.pixmap_formats_len as usize)?;

        let formats: &[PixmapFormat] = request::decode_slice(&bytes, response.pixmap_formats_len as usize);

        // println!("formats: {:?}", formats);

        for _ in 0..response.roots_len {
            let mut screen = Screen::new(self.stream.recv_decode()?);

            for _ in 0..screen.response.allowed_depths_len {
                let mut depth = Depth::new(self.stream.recv_decode()?);

                let bytes = self.stream.recv(std::mem::size_of::<VisualResponse>() * depth.length as usize)?;

                depth.extend(request::decode_slice(&bytes, depth.length as usize));

                screen.depths.push(depth);
            }

            self.roots.push(screen);
        }

        let stream = self.stream.try_clone()?;
        let events = self.events.clone();
        let replies = self.replies.clone();
        let sequence = self.sequence.clone();
        let roots = self.roots.clone();

        thread::spawn(move || {
            let mut listener = EventListener::new(stream, events, replies, sequence, roots);

            if let Err(err) = listener.listen() {
                println!("[ERROR] listener failed: {}", err);
            }
        });

        xid::setup(response.resource_id_base, response.resource_id_mask)?;

        Ok(())
    }

    fn setup<'a>(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let entry = auth::entry()?;

        let request = SetupRequest::new(self.endian(), X_PROTOCOL, X_PROTOCOL_REVISION, entry.name.len() as u16, entry.data.len() as u16);

        self.stream.send(request::encode(&request))?;

        self.stream.send_arr(&[entry.name.clone(), vec![0u8; request::pad(entry.name.len())], entry.data.clone(), vec![0u8; request::pad(entry.data.len())]])?;

        let response: SetupResponse = self.stream.recv_decode()?;

        match response.status {
            1 => self.read_setup(),
            0 => Err(Box::new(Error::SetupFailed { reason: self.stream.recv_str(response.padding as usize)? })),
            2 => Err(Box::new(Error::Authenthicate)),
            _ => Err(Box::new(Error::InvalidStatus)),
        }
    }
}

pub struct EventListener<T: Send + Sync + Read + Write + TryClone> {
    stream: Stream<T>,
    events: Queue<Event>,
    replies: Queue<Reply>,
    sequence: SequenceManager,
    roots: Roots,
}

impl<T> EventListener<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, events: Queue<Event>, replies: Queue<Reply>, sequence: SequenceManager, roots: Roots) -> EventListener<T> {
        EventListener {
            stream,
            events,
            replies,
            sequence,
            roots,
        }
    }

    fn handle_reply(&mut self, event: GenericEvent) -> Result<(), Box<dyn std::error::Error>> {
        let sequence = self.sequence.get(event.sequence)?;

        match sequence.kind {
            ReplyKind::InternAtom => {
                let response: InternAtomResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::InternAtom(response))?;
            },
            ReplyKind::GetWindowAttributes => {
                let response: GetWindowAttributesResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetWindowAttributes(response))?;
            },
            ReplyKind::QueryPointer => {
                let response: QueryPointerResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::QueryPointer(response))?;
            },
            ReplyKind::GetProperty => {
                let response: GetPropertyResponse = self.stream.recv_decode()?;

                self.replies.push(Reply::GetProperty {
                    value: self.stream.recv(response.value_len as usize)?,
                })?;

                self.stream.recv(request::pad(response.value_len as usize))?;
            },
        }

        Ok(())
    }

    fn handle_event(&mut self, generic: GenericEvent) -> Result<(), Box<dyn std::error::Error>> {
        match generic.opcode & 0b0111111 {
            Response::ERROR => {
                let error: ErrorEvent = self.stream.recv_decode()?;

                println!("error: {:?}", error);

                Err(Box::new(Error::Event {
                    detail: generic.detail,
                    sequence: generic.sequence,
                }))
            },
            Response::REPLY => {
                self.handle_reply(generic)?;

                Ok(())
            },
            Response::KEY_PRESS | Response::KEY_RELEASE => {
                let key_event: KeyEvent = self.stream.recv_decode()?;

                self.events.push(Event::KeyEvent {
                    kind: match generic.opcode & 0b0111111 {
                        Response::KEY_PRESS => KeyEventKind::Press,
                        Response::KEY_RELEASE => KeyEventKind::Release,
                        _ => unreachable!(),
                    },
                    coordinates: Coordinates::new(key_event.event_x, key_event.event_y, key_event.root_x, key_event.root_y),
                    window: key_event.event,
                    root: key_event.root,
                    subwindow: key_event.child,
                    state: key_event.state,
                    keycode: generic.detail,
                    send_event: key_event.same_screen == 0,
                })
            },
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
            },
            Response::DESTROY_NOTIFY => {
                let event: DestroyNotify = self.stream.recv_decode()?;

                self.events.push(Event::DestroyNotify {
                    event: event.event,
                    window: event.window,
                })
            },
            Response::UNMAP_NOTIFY => {
                let event: UnmapNotify = self.stream.recv_decode()?;

                self.events.push(Event::UnmapNotify {
                    event: event.event,
                    window: event.window,
                    configure: event.from_configure == 0,
                })
            },
            Response::MAP_NOTIFY => {
                let event: MapNotify = self.stream.recv_decode()?;

                self.events.push(Event::MapNotify {
                    event: event.event,
                    window: event.window,
                    override_redirect: event.override_redirect == 0,
                })
            },
            Response::MAP_REQUEST => {
                let event: MapReq = self.stream.recv_decode()?;

                self.events.push(Event::MapRequest {
                    parent: event.parent,
                    window: event.window,
                })
            },
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
            },
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
            },
            Response::CONFIGURE_REQUEST => {
                let event: ConfigReq = self.stream.recv_decode()?;

                self.events.push(Event::ConfigureRequest {
                    stack_mode: StackMode::from(generic.detail),
                    parent: event.parent,
                    window: event.window,
                    sibling: event.sibling,
                    x: event.x,
                    y: event.y,
                    width: event.height,
                    height: event.height,
                    border_width: event.border_width,
                    mask: event.value_mask,
                })
            },
            Response::GRAVITY_NOTIFY => {
                let event: GravityNotify = self.stream.recv_decode()?;

                self.events.push(Event::GravityNotify {
                    event: event.event,
                    window: event.window,
                    x: event.x,
                    y: event.y,
                })
            },
            Response::CIRCULATE_NOTIFY => {
                let event: CircNotify = self.stream.recv_decode()?;

                self.events.push(Event::CirculateNotify {
                    event: event.event,
                    window: event.window,
                    place: Place::from(event.place),
                })
            },
            Response::CIRCULATE_REQUEST => {
                let event: CircReq = self.stream.recv_decode()?;

                self.events.push(Event::CirculateRequest {
                    parent: event.event,
                    window: event.window,
                    place: Place::from(event.place),
                })
            },
            Response::SELECTION_CLEAR => {
                let event: SelectionClear = self.stream.recv_decode()?;

                self.events.push(Event::SelectionClear {
                    time: event.time,
                    owner: event.owner,
                    selection: Atom::new(event.selection),
                })
            },
            Response::SELECTION_REQUEST => {
                let event: SelectionReq = self.stream.recv_decode()?;

                self.events.push(Event::SelectionRequest {
                    time: event.time,
                    owner: event.requestor,
                    selection: Atom::new(event.selection),
                    target: Atom::new(event.target),
                    property: Atom::new(event.property),
                })
            },
            Response::SELECTION_NOTIFY => {
                let event: SelectionNotify = self.stream.recv_decode()?;

                self.events.push(Event::SelectionNotify {
                    time: event.time,
                    requestor: event.requestor,
                    selection: Atom::new(event.selection),
                    target: Atom::new(event.target),
                    property: Atom::new(event.property),
                })
            },
            Response::CLIENT_MESSAGE => {
                let event: CircReq = self.stream.recv_decode()?;

                self.events.push(Event::CirculateRequest {
                    parent: event.event,
                    window: event.window,
                    place: Place::from(event.place),
                })
            },
            Response::MAPPING_NOTIFY => {
                let event: MappingNotify = self.stream.recv_decode()?;

                self.events.push(Event::MappingNotify {
                    request: event.request,
                    keycode: event.keycode,
                    count: event.count,
                })
            },
            _ => Ok(()),
        }
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let event: GenericEvent = self.stream.recv_decode()?;

            self.handle_event(event)?;
        }
    }
}

pub fn open_tcp<'a>(display: u16) -> Result<Display<TcpStream>, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], X_TCP_PORT + display)))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}

pub fn open_unix<'a>(display: u16) -> Result<Display<UnixStream>, Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(format!("/tmp/.X11-unix/X{}", display))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}


