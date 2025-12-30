# Yaxi

yaxi is a x11 library written from scratch

![GitHub License](https://img.shields.io/badge/license-MIT-red?style=for-the-badge&logo=none)
![dependencies](https://deps.rs/repo/github/proxin187/yaxi/status.svg?style=for-the-badge)
[![crates.io](https://img.shields.io/badge/crates.io-yaxi-red?style=for-the-badge&logo=none)](https://crates.io/crates/yaxi)


## Key Features

* Clean Interface - yaxi provides a clean interface, making it perfect for both beginners and experienced developers
* Safety - yaxi has safe interface for x11 unlike many other x11 libraries
* Not A Wrapper - yaxi is a pure rust implementation and is NOT a wrapper
* No Release Dependencies - yaxi release builds dont depend on any crates

## Goals
- [X] Authorization
- [X] Requests and Replies
- [X] events (most)
- [X] Keycodes and Keysyms
- [X] Extensions (Xinerama, Xft, ...) (Only Xinerama Implemented So Far)
- [X] Clipboard Support
- [ ] Comprehensive Documentation
- [ ] GLX
- [ ] Window Manager in yaxi (work in progress)

## Usage
This crate is on [crates.io](https://crates.io/crates/yaxi) and can be added either through
adding `yaxi` to your dependencies in `Cargo.toml`:
```toml
[dependencies]
yaxi = "0.1.57"
```
Or running the following Cargo command in your project directory:
```bash
cargo add yaxi
```

## Example

This example opens a window and waits for a keyboard press before it quits:

```rust
use yaxi::window::{WindowArguments, ValuesBuilder, WindowKind};
use yaxi::proto::{Event, WindowClass, EventMask};
use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open(None)?;

    let mut root = display.default_root_window()?;

    let mut window = root.create_window(WindowArguments {
        depth: root.depth(),
        x: 5,
        y: 5,
        width: 80,
        height: 50,
        border_width: 15,
        class: WindowClass::InputOutput,
        visual: root.visual(),
        values: ValuesBuilder::new(vec![]),
    })?;

    window.select_input(&[EventMask::KeyPress, EventMask::KeyRelease])?;

    window.map(WindowKind::Window)?;

    let event = display.next_event()?;

    match event {
        Event::KeyEvent { kind, coordinates, window, root, subwindow, state, keycode, send_event } => {
            let window_copy = display.window_from_id(window)?;

            println!("window from id: {}, keycode: {}", window_copy.id(), keycode);
        },
        _ => {},
    }

    window.destroy(WindowKind::Window)?;

    Ok(())
}


```

## License

Yaxi is licensed under the MIT License.


