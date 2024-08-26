use xrs::display::window::{WindowArguments, WindowValuesBuilder, WindowClass, VisualClass};
use xrs::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = display::open_unix(2)?;

    let mut root = display.default_root_window()?;

    let window = root.create_window(WindowArguments {
        depth: 0,
        x: 0,
        y: 0,
        width: 100,
        height: 200,
        border_width: 15,
        class: WindowClass::InputOutput,
        visual: VisualClass::TrueColor,
        values: WindowValuesBuilder::new(&[]),
    })?;

    loop {}

    Ok(())
}


