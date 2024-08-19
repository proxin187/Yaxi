use std::slice;
use std::mem;
use std::ptr;

pub enum Event {
}

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
    pub fn new(endian: u8, major_version: u16, minor_version: u16) -> SetupRequest {
        SetupRequest {
            endian,
            pad0: 0,
            major_version,
            minor_version,
            name_len: 0,
            data_len: 0,
            pad1: [0; 2],
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
#[derive(Debug)]
pub struct Screen {
    root: u32,
    default_colormap: u32,
    white_pixel: u32,
    black_pixel: u32,
    current_input_mask: u32,
    width_in_pixels: u16,
    height_in_pixels: u16,
    width_in_mm: u16,
    height_in_mm: u16,
    min_installed_maps: u16,
    max_installed_maps: u16,
    root_visual: u32,
    backing_stores: u8,
    save_unders: u8,
    root_depth: u8,
    allowed_depths_len: u8,
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


