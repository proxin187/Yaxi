use xrs::display::window::{WindowArguments, WindowValuesBuilder, WindowClass, WindowKind, VisualClass};
use xrs::display::proto::ReplyKind;
use xrs::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    display.intern_atom("WM_CLASS", true)?;

    let reply = display.wait_for_reply()?;

    println!("reply: {:?}", reply);

    /*
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

    window.map(WindowKind::Window)?;

    for _ in 0..99999999u64 {}

    window.destroy(WindowKind::Window)?;
    */

    Ok(())
}


