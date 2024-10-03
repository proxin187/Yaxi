use yaxi::window::{PropFormat, PropMode, WindowArguments, WindowValuesBuilder, EventMask, WindowValue, WindowClass, WindowKind, VisualClass};
use yaxi::proto::{Event, KeyEventKind};
use yaxi::display::{self, Atom};

// NOTE: this is just a test bench for testing features


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    let mut root = display.default_root_window()?;

    println!("root_depth: {}", root.depth());
    println!("root_visual: {:?}", root.visual());

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

    /*
    println!("getting window from id");

    let mut window = display.window_from_id(window.id())?;

    println!("window from id: {}", window.id());
    */

    let atom = display.intern_atom("TEST2", false)?;

    println!("id: {:?}", atom.id());

    println!("property: {:?}", window.get_property(atom, Atom::CARDINAL, false)?);

    window.change_property(atom, Atom::CARDINAL, PropFormat::Format8, PropMode::Replace, &[2])?;

    println!("property: {:?}", window.get_property(atom, Atom::CARDINAL, false)?);

    window.select_input(&[EventMask::KeyPress, EventMask::KeyRelease])?;

    window.map(WindowKind::Window)?;

    let event = display.next_event()?;

    println!("event: {:?}", event);

    match event {
        Event::KeyEvent { kind, coordinates, window, root, subwindow, state, keycode, send_event } => {
            let mut window_copy = display.window_from_id(window)?;

            println!("window from id: {}, keycode: {}", window_copy.id(), keycode);

            let keysym = display.keysym_from_keycode(keycode)?;

            println!("keysym: {:?}", keysym);
            println!("character: {:?}", keysym.character()?);
        },
        _ => {},
    }

    let pointer = window.query_pointer()?;

    println!("pointer: {:?}", pointer);

    window.destroy(WindowKind::Window)?;

    /*
    window.map(WindowKind::Window)?;

    for _ in 0..99999999u64 {}

    window.destroy(WindowKind::Window)?;
    */

    Ok(())
}


