use super::*;


pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

#[derive(Clone, Copy)]
pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PsuedoColor = 3,
    TrueColor = 4,
    DirectColor = 33,
}

impl From<u32> for VisualClass {
    fn from(value: u32) -> VisualClass {
        println!("value: {}", value);

        match value {
            0 => VisualClass::StaticGray,
            1 => VisualClass::GrayScale,
            2 => VisualClass::StaticColor,
            3 => VisualClass::PsuedoColor,
            4 => VisualClass::TrueColor,
            33 => VisualClass::DirectColor,
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
            .fold(0, |acc, x| acc ^ x)
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
    pub visual: VisualClass,
    pub values: WindowValuesBuilder,
}

pub struct Window<T> {
    stream: Stream<T>,
    visual: VisualClass,
    depth: u8,
    id: u32,
}

impl<T> Window<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, visual: VisualClass, depth: u8, id: u32) -> Window<T> {
        Window {
            stream,
            visual,
            depth,
            id,
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn depth(&self) -> u8 { self.depth }

    pub fn visual(&self) -> VisualClass { self.visual }

    // TODO: this function is the problem
    // its most likely related to window values

    pub fn create_window(&mut self, mut window: WindowArguments) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let window_values_request = window.values.build()?;

        let window_request = CreateWindow {
            opcode: Opcode::CREATE_WINDOW,
            depth: window.depth,
            length: 8 + window.values.len(),
            wid: xid::next()?,
            parent: self.id(),
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            border_width: window.border_width,
            class: window.class as u16,
            visual: window.visual as u32,
            value_mask: window.values.value_mask,
        };

        self.stream.inner.write_all(request::encode(&window_request))?;

        // TODO: it only seems like it writes the arguments it knows its going to use
        // only the arguments u use are sendt

        // self.stream.inner.write_all(request::encode(&window_values_request))?;

        // TODO: SEQUENCE THINGY

        Ok(Window::new(self.stream.try_clone()?, window.visual, window_request.depth, window_request.wid))
    }

    pub fn grab_key(&mut self) {
    }
}


