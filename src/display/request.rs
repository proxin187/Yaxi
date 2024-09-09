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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct InternAtomResponse {
    pub length: u32,
    pub atom: u32,
    pub pad0: [u8; 20],
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct GetWindowAttributesResponse {
    pub length: u32,
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
    pub pad0: [u8; 2],
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
pub struct GrabKey {
    major_opcode: u8,
    owner_events: u8,
    length: u16,
    grab_window: u32,
    modifiers: u16,
    key: u8,
    pointer_mode: u8,
    keyboard_mode: u8,
    pad0: [u8; 3],
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


