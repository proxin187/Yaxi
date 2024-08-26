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
    OverrideRedirect(bool),
    SaveUnder(bool),
    EventMask(Vec<EventMask>),
    DoNotPropogateMask(Vec<EventMask>),
    Colormap(u32),
    Cursor(Cursor),
}

pub struct WindowValuesBuilder {
    values: Vec<WindowValue>,
    request: WindowValuesRequest,
}

impl WindowValuesBuilder {
    pub fn new(values: &[WindowValue]) -> WindowValuesBuilder {
        WindowValuesBuilder {
            values: values.to_vec(),
            request: WindowValuesRequest::default(),
        }
    }

    fn mask(&self, masks: Vec<EventMask>) -> u32 {
        masks.iter()
            .map(|event_mask| *event_mask as u32)
            .fold(0, |acc, x| acc ^ x)
    }

    fn insert_value(&mut self, value: WindowValue) {
        match value {
            WindowValue::BgPixmap(pixmap) => self.request.background_pixmap = pixmap,
            WindowValue::BgPixel(pixel) => self.request.background_pixel = pixel,
            WindowValue::BorderPixmap(pixmap) => self.request.border_pixmap = pixmap,
            WindowValue::BorderPixel(pixel) => self.request.border_pixel = pixel,
            WindowValue::BitGravity(gravity) => self.request.bit_gravity = gravity as u32,
            WindowValue::WinGravity(gravity) => self.request.win_gravity = gravity as u32,
            WindowValue::BackingStore(store) => self.request.backing_store = store as u32,
            WindowValue::BackingPlane(plane) => self.request.backing_plane = plane as u32,
            WindowValue::OverrideRedirect(value) => self.request.override_redirect = value as u8,
            WindowValue::SaveUnder(value) => self.request.save_under = value as u8,
            WindowValue::EventMask(masks) => self.request.event_mask = self.mask(masks),
            WindowValue::DoNotPropogateMask(masks) => self.request.do_not_propogate_mask = self.mask(masks),
            WindowValue::Colormap(colormap) => self.request.colormap = colormap,
            WindowValue::Cursor(cursor) => self.request.cursor = cursor as u32,
        }
    }

    pub fn build(&mut self) -> Result<WindowValuesRequest, Box<dyn std::error::Error>> {
        for value in self.values.clone() {
            self.insert_value(value);
        }

        Ok(self.request)
    }
}

pub struct WindowArguments {
    pub depth: u8,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub class: WindowClass,
    pub visual: VisualClass,
    pub values: WindowValuesBuilder,
}

pub struct Window<T> {
    stream: Stream<T>,
    id: u32,
}

impl<T> Window<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, id: u32) -> Window<T> {
        Window {
            stream,
            id,
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn create_window(&mut self, mut window: WindowArguments) -> Result<Window<T>, Box<dyn std::error::Error>> {
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

        let window_values_request = window.values.build()?;

        self.stream.inner.write_all(request::encode(&window_values_request))?;

        Ok(Window::new(self.stream.try_clone()?, window_request.wid))
    }

    pub fn grab_key(&mut self) {
    }
}


