use super::*;


pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PsuedoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

pub enum BackingStore {
    NotUseful = 0,
    WhenMapped = 1,
    Always = 2,
}

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

// TODO: FINISH THIS
pub enum WindowValue<'a> {
    BgPixmap(u32),
    BgPixel(u32),
    BorderPixmap(u32),
    BorderPixel(u32),
    BitGravity(Gravity),
    WinGravity(Gravity),
    BackingStore(BackingStore),
    BackingPlane(u32),
    OverrideRedirect(bool),
    SaveUnder(bool),
    EventMask(&'a [EventMask]),
}

pub struct WindowArguments {
    depth: u8,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    border_width: u16,
    class: WindowClass,
    visual: VisualClass,
}

pub struct Window<T> {
    stream: Stream<T>,
    id: u32,
}

impl<T> Window<T> where T: Send + Sync + Read + Write + TryClone {
    fn new(inner: T, id: u32) -> Window<T> {
        Window {
            stream: Stream::new(inner),
            id,
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn create_window(&mut self, window: WindowArguments, values: &[WindowValue]) -> Result<(), Box<dyn std::error::Error>> {
        let window_request = CreateWindow {
            opcode: Opcode::CREATE_WINDOW,
            depth: window.depth,
            length: 0,
            wid: xid::next()?,
            parent: self.id(),
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            border_width: window.border_width,
            class: window.class as u16,
            visual: window.visual as u32,
        };

        self.stream.inner.write_all(request::encode(&window_request))?;

        Ok(())
    }

    pub fn grab_key(&mut self) {
    }
}


