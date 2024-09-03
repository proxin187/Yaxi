use xrs::display::window::{WindowArguments, WindowValuesBuilder, WindowClass, WindowKind, VisualClass};
use xrs::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = display::open_unix(0)?;

    let mut root = display.default_root_window()?;

    println!("root_depth: {}", root.depth());

    // TODO: properly implement Visual
    let mut window = root.create_window(WindowArguments {
        depth: root.depth(),
        x: 5,
        y: 5,
        width: 80,
        height: 50,
        border_width: 15,
        class: WindowClass::InputOutput,
        visual: VisualClass::StaticGray,
        values: WindowValuesBuilder::new(&[]),
    })?;

    window.map(WindowKind::Window)?;

    for _ in 0..99999999u64 {}

    window.destroy(WindowKind::Window)?;

    Ok(())
}


