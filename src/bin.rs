use xrs::display::window::{PropFormat, PropMode, WindowArguments, WindowValuesBuilder, EventMask, WindowValue, WindowClass, WindowKind, VisualClass};
use xrs::display::{self, Atom};


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
        values: WindowValuesBuilder::new(&[WindowValue::EventMask(vec![EventMask::KeyPress, EventMask::KeyRelease])]),
    })?;

    let atom = display.intern_atom("TEST2", false)?;

    println!("id: {:?}, name: {:?}", atom.id(), atom.name());

    println!("property: {:?}", window.get_property(atom, Atom::CARDINAL, false)?);

    window.change_property(atom, Atom::CARDINAL, PropFormat::Format8, PropMode::Replace, &[2])?;

    println!("property: {:?}", window.get_property(atom, Atom::CARDINAL, false)?);

    /*
    window.map(WindowKind::Window)?;

    for _ in 0..99999999u64 {}

    window.destroy(WindowKind::Window)?;
    */

    Ok(())
}


