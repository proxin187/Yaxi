use yaxi::display;
use yaxi::proto::{Event, EventMask, WindowClass};
use yaxi::window::{ValuesBuilder, WindowArguments, WindowKind};

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
        Event::KeyEvent {
            kind,
            coordinates,
            window,
            root,
            subwindow,
            state,
            keycode,
            send_event,
        } => {
            let window_copy = display.window_from_id(window)?;

            println!("window from id: {}, keycode: {}", window_copy.id(), keycode);
        }
        _ => {}
    }

    window.destroy(WindowKind::Window)?;

    Ok(())
}
