use std::slice;
use std::mem;
use std::ptr;


// https://xcb.freedesktop.org/manual/xproto_8h_source.html

#[repr(packed, C)]
pub struct SetupRequest {
    endian: u8,
    pad0: u8,
    major_version: u16,
    minor_version: u16,
    name_len: u16,
    data_len: u16,
    pad1: [u8; 2],
}

impl SetupRequest {
    pub fn new(endian: u8, major_version: u16, minor_version: u16, name_len: u16, data_len: u16) -> SetupRequest {
        SetupRequest {
            endian,
            pad0: endian,
            major_version,
            minor_version,
            name_len,
            data_len,
            pad1: [endian; 2],
        }
    }
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct SetupResponse {
    pub status: u8,
    pub padding: u8,
    pub major_version: u16,
    pub minor_version: u16,
    pub length: u16,
}

#[repr(packed, C)]
#[derive(Debug, Default)]
pub struct SuccessResponse {
    pub release_number: u32,
    pub resource_id_base: u32,
    pub resource_id_mask: u32,
    pub motion_buffer_size: u32,
    pub vendor_len: u16,
    pub maximum_request_len: u16,
    pub roots_len: u8,
    pub pixmap_formats_len: u8,
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
    pub padding: [u8; 4],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct PixmapFormat {
    depth: u8,
    bits_per_pixel: u8,
    scanline_pad: u8,
    padding: [u8; 5],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct ScreenResponse {
    pub root: u32,
    pub default_colormap: u32,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_mask: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_mm: u16,
    pub height_in_mm: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: u32,
    pub backing_stores: u8,
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths_len: u8,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct DepthResponse {
    pub depth: u8,
    pub pad0: u8,
    pub visuals_len: u16,
    pub pad1: [u8; 4],
}

#[repr(packed, C)]
#[derive(Debug, Clone, Copy)]
pub struct VisualResponse {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pad0: [u8; 4],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GenericEvent {
    pub opcode: u8,
    pub detail: u8,
    pub sequence: u16,
    // pub length: u32,
    // pub event_type: u16,
    // pub pad0: [u8; 22],
    // pub full_sequence: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ErrorEvent {
    pub bad_value: u32,
    pub minor_opcode: u16,
    pub major_opcode: u8,
    pub pad0: u8,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct KeyEvent {
    pub time: u32,
    pub root: u32,
    pub event: u32,
    pub child: u32,
    pub root_x: u16,
    pub root_y: u16,
    pub event_x: u16,
    pub event_y: u16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ButtonEvent {
    pub time: u32,
    pub root: u32,
    pub event: u32,
    pub child: u32,
    pub root_x: u16,
    pub root_y: u16,
    pub event_x: u16,
    pub event_y: u16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

pub type MotionNotify = ButtonEvent;

#[repr(packed, C)]
#[derive(Debug)]
pub struct CircNotify {
    pub event: u32,
    pub window: u32,
    pub unused: u32,
    pub place: u8,
    pub pad0: [u8; 15],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ConfigNotify {
    pub event: u32,
    pub window: u32,
    pub above_sibling: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: u8,
    pub pad0: [u8; 5],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct CreateNotify {
    pub event: u32,
    pub window: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: u8,
    pub pad0: [u8; 9],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct DestroyNotify {
    pub event: u32,
    pub window: u32,
    pub pad0: [u8; 20],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GravityNotify {
    pub event: u32,
    pub window: u32,
    pub x: u16,
    pub y: u16,
    pub pad0: [u8; 16],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct MapNotify {
    pub event: u32,
    pub window: u32,
    pub override_redirect: u8,
    pub pad0: [u8; 19],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ReparentNotify {
    pub event: u32,
    pub window: u32,
    pub parent: u32,
    pub x: u16,
    pub y: u16,
    pub override_redirect: u8,
    pub pad0: [u8; 11],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct UnmapNotify {
    pub event: u32,
    pub window: u32,
    pub from_configure: u8,
    pub pad0: [u8; 19],
}

pub type CircReq = CircNotify;

#[repr(packed, C)]
#[derive(Debug)]
pub struct ConfigReq {
    pub parent: u32,
    pub window: u32,
    pub sibling: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub value_mask: u16,
    pub pad0: [u8; 4],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct MapReq {
    pub parent: u32,
    pub window: u32,
    pub pad0: [u8; 20],
}

// TODO: IMPLEMENT CLIENT MESSAGE

#[repr(packed, C)]
#[derive(Debug)]
pub struct ClientMessage {
    pub window: u32,
    pub type_: u32,
    pub data: [u8; 20],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct MappingNotify {
    pub request: u8,
    pub keycode: u8,
    pub count: u8,
    pub pad0: [u8; 25],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct SelectionClear {
    pub time: u32,
    pub owner: u32,
    pub selection: u32,
    pub pad0: [u8; 16],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct SelectionNotify {
    pub time: u32,
    pub requestor: u32,
    pub selection: u32,
    pub target: u32,
    pub property: u32,
    pub pad0: [u8; 8],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct SelectionReq {
    pub time: u32,
    pub owner: u32,
    pub requestor: u32,
    pub selection: u32,
    pub target: u32,
    pub property: u32,
    pub pad0: [u8; 4],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct EnterNotify {
    pub time: u32,
    pub root: u32,
    pub event: u32,
    pub child: u32,
    pub root_x: u16,
    pub root_y: u16,
    pub event_x: u16,
    pub event_y: u16,
    pub state: u16,
    pub mode: u8,
    pub sf: u8,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct CreateWindow {
    pub opcode: u8,
    pub depth: u8,
    pub length: u16,
    pub wid: u32,
    pub parent: u32,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub class: u16,
    pub visual: u32,
    pub value_mask: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GenericWindow {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub wid: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct InternAtom {
    pub opcode: u8,
    pub only_if_exists: u8,
    pub length: u16,
    pub name_len: u16,
    pub pad1: [u8; 2],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct InternAtomResponse {
    pub(crate) length: u32,
    pub atom: u32,
    pub(crate) pad0: [u8; 20],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ChangeProperty {
    pub opcode: u8,
    pub mode: u8,
    pub length: u16,
    pub window: u32,
    pub property: u32,
    pub type_: u32,
    pub format: u8,
    pub pad0: [u8; 3],
    pub data_len: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GetProperty {
    pub opcode: u8,
    pub delete: u8,
    pub length: u16,
    pub window: u32,
    pub property: u32,
    pub type_: u32,
    pub long_offset: u32,
    pub long_length: u32,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetPropertyResponse {
    pub length: u32,
    pub type_: u32,
    pub bytes_after: u32,
    pub value_len: u32,
    pub pad0: [u8; 12],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ReparentWindow {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u32,
    pub window: u32,
    pub parent: u32,
    pub x: u16,
    pub y: u16,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GetWindowAttributes {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub wid: u32,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetWindowAttributesResponse {
    pub(crate) length: u32,
    pub visual: u32,
    pub class: u16,
    pub bit_gravity: u8,
    pub win_gravity: u8,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub save_under: u8,
    pub map_is_installed: u8,
    pub map_state: u8,
    pub override_redirect: u8,
    pub colormap: u32,
    pub all_event_mask: u32,
    pub your_event_mask: u32,
    pub do_not_propogate_mask: u16,
    pub(crate) pad0: [u8; 2],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ChangeWindowAttributes {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub wid: u32,
    pub mask: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct QueryPointer {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub wid: u32,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct QueryPointerResponse {
    pub(crate) length: u32,
    pub root: u32,
    pub child: u32,
    pub root_x: u16,
    pub root_y: u16,
    pub win_x: u16,
    pub win_y: u16,
    pub mask: u16,
    pub(crate) pad0: [u8; 6],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetKeyboardMapping {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub first: u8,
    pub count: u8,
    pub pad1: [u8; 2],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct KeyboardMappingResponse {
    pub length: u32,
    pub pad0: [u8; 24],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GrabKey {
    pub opcode: u8,
    pub owner_events: u8,
    pub length: u16,
    pub grab_window: u32,
    pub modifiers: u16,
    pub key: u8,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub pad0: [u8; 3],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GrabButton {
    pub opcode: u8,
    pub owner_events: u8,
    pub length: u16,
    pub grab_window: u32,
    pub event_mask: u16,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub confine_to: u32,
    pub cursor: u32,
    pub button: u8,
    pub pad0: u8,
    pub modifiers: u16,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct UngrabButton {
    pub opcode: u8,
    pub button: u8,
    pub length: u16,
    pub grab_window: u32,
    pub modifiers: u16,
    pub pad0: [u8; 2],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GrabPointer {
    pub opcode: u8,
    pub owner_events: u8,
    pub length: u16,
    pub grab_window: u32,
    pub event_mask: u16,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub confine_to: u32,
    pub cursor: u32,
    pub time: u32,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GrabPointerResponse {
    pub length: u32,
    pub(crate) pad0: [u8; 24],
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct UngrabPointer {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub time: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct ConfigureWindow {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub wid: u32,
    pub mask: u16,
    pub pad1: u16,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct KillClient {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub resource: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct SetInputFocus {
    pub opcode: u8,
    pub revert_to: u8,
    pub length: u16,
    pub focus: u32,
    pub time: u32,
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct GetInputFocus {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetInputFocusResponse {
    pub(crate) length: u32,
    pub window: u32,
    pub(crate) pad0: [u8; 20],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct FocusIn {
    pub event: u32,
    pub mode: u8,
    pub(crate) pad0: [u8; 23],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct FocusOut {
    pub event: u32,
    pub mode: u8,
    pub(crate) pad0: [u8; 23],
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetGeometry {
    pub opcode: u8,
    pub pad0: u8,
    pub length: u16,
    pub window: u32,
}

#[repr(packed, C)]
#[derive(Debug, Clone)]
pub struct GetGeometryResponse {
    pub(crate) length: u32,
    pub root: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub(crate) pad0: [u8; 10],
}

pub fn encode<T>(ptr: &T) -> &[u8] {
    unsafe {
        slice::from_raw_parts((ptr as *const T) as *const u8, mem::size_of::<T>())
    }
}

pub fn decode<'a, T>(bytes: &'a [u8]) -> T {
    unsafe {
        assert_eq!(bytes.len(), mem::size_of::<T>());

        ptr::read(bytes.as_ptr() as *const T)
    }
}

pub fn decode_slice<'a, T>(bytes: &'a [u8], length: usize) -> &'a [T] {
    unsafe {
        assert_eq!(bytes.len(), mem::size_of::<T>() * length);

        std::slice::from_raw_parts(bytes.as_ptr() as *const T, length)
    }
}

pub fn pad(len: usize) -> usize {
    (4 - (len % 4)) % 4
}


