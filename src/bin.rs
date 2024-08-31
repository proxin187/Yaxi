use xrs::display::window::{WindowArguments, WindowValuesBuilder, WindowClass, VisualClass};
use xrs::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = display::open_unix(2)?;

    let mut root = display.default_root_window()?;

    println!("root_depth: {}", root.depth());

    // TODO: create_window is the problem

    let window = root.create_window(WindowArguments {
        depth: root.depth(),
        x: 0,
        y: 0,
        width: 100,
        height: 200,
        border_width: 15,
        class: WindowClass::InputOutput,
        // visual: root.visual(),
        visual: VisualClass::DirectColor,
        values: WindowValuesBuilder::new(&[]),
    })?;

    loop {}

    Ok(())
}


