<h1 align="center">
    <br>
    <a href="https://github.com/proxin187/yaxi">
        <img src="assets/logo1.png" width="400">
    </a>
    <br>
</h1>

<h4 align="center">yaxi is a x11 library written from scratch</h4>

> [!WARNING]
> yaxi is a work-in-progress library and breaking changes may occur.

## Key Features

* Clean Interface - yaxi provides a clean interface, making it perfect for both beginners and experienced developers
* Safety - yaxi has safe interface for x11 unlike many other x11 libraries
* Not A Wrapper - yaxi is a pure rust implementation and is NOT a wrapper

## Example

This example opens a window and waits for a keyboard press before it quits:

```rust
use yaxi::window::{WindowArguments, WindowValuesBuilder, EventMask, WindowClass, WindowKind};
use yaxi::proto::Event;
use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

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
        values: WindowValuesBuilder::new(&[]),
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

## Roadmap

- [X] Authorization
- [X] Requests and Replies
- [X] events (most)
- [ ] Window Manager in yaxi (work in progress)

## License

Yaxi is licensed under the MIT License.


