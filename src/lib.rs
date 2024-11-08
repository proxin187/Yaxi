//! This crate provides a high level interface to the x11 [protocol] in pure Rust.
//!
//! [protocol]: https://www.x.org/docs/XProtocol/proto.pdf
//!
//! # Features
//! - Clean Interface - yaxi provides a clean interface, making it perfect for both beginners and experienced developers
//! - Safety - yaxi has safe interface for x11 unlike many other x11 libraries
//! - Not A Wrapper - yaxi is a pure rust implementation and is NOT a wrapper
//! - No Dependencies - yaxi doesnt depend on any crates
//!
//! # Usage
//! This crate is on [crates.io](https://crates.io/crates/yaxi) and can be added either through
//! adding `yaxi` to your dependencies in `Cargo.toml`:
//! ```toml
//! [dependencies]
//! yaxi = "0.1.45"
//! ```
//!
//! Or running the following Cargo command in your project directory:
//! ```
//! cargo add yaxi
//! ```
//!
//! # [Example: open a window](https://github.com/proxin187/Yaxi/tree/main/examples/minimal)
//! The following example opens a window and waits for a keyboard press before it quits:
//!
//! ```rust
//! use yaxi::window::{WindowArguments, ValuesBuilder, WindowKind};
//! use yaxi::proto::{Event, WindowClass, EventMask};
//! use yaxi::display;
//!
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut display = display::open(None)?;
//!
//!     let mut root = display.default_root_window()?;
//!
//!     let mut window = root.create_window(WindowArguments {
//!         depth: root.depth(),
//!         x: 5,
//!         y: 5,
//!         width: 80,
//!         height: 50,
//!         border_width: 15,
//!         class: WindowClass::InputOutput,
//!         visual: root.visual(),
//!         values: ValuesBuilder::new(vec![]),
//!     })?;
//!
//!     window.select_input(&[EventMask::KeyPress, EventMask::KeyRelease])?;
//!
//!     window.map(WindowKind::Window)?;
//!
//!     let event = display.next_event()?;
//!
//!     match event {
//!         Event::KeyEvent { kind, coordinates, window, root, subwindow, state, keycode, send_event } => {
//!             let window_copy = display.window_from_id(window)?;
//!
//!             println!("window from id: {}, keycode: {}", window_copy.id(), keycode);
//!         },
//!         _ => {},
//!     }
//!
//!     window.destroy(WindowKind::Window)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note that the functions are appropriately mapped to their respective structure eg. (Window, Display), a feature shy from [most other x11 libraries]().
//!
//! For more examples please visit the [repo](https://github.com/proxin187/Yaxi/tree/main/examples)
//!
//! # Crate features
//! By default [yaxi](https://github.com/proxin187/Yaxi) only has the standard x11 protocol, but
//! with the following crate features the user gets additional functionality:
//! - `xinerama` - this feature allows the user to interface with [Xinerama](https://en.wikipedia.org/wiki/Xinerama), an extension to the x11 protocol enabling to use two or more physical displays as one shared display
//! - `clipboard` - extensible builtin clipboard functionality
//! - `extras` - enables some convencience functions that arent a part of the official protocol
//!

pub mod display;

/// proto contains protocol specific code such as opcodes, replies and so on.
pub mod proto;

/// window contains the core functionality for handling windows
pub mod window;

/// keyboard contains keysyms and keycodes for x11
pub mod keyboard;

/// implementation of popular x11 extensions such as xinerama
pub mod extension;

/// clipboard is a user-friendly wrapper around x11 selections

#[cfg(feature = "clipboard")]
pub mod clipboard;
