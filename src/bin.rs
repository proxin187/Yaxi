use xrs::display::window::{WindowArguments, WindowValuesBuilder, WindowClass, VisualClass};
use xrs::display::Authenthication;
use xrs::display;
use xrs::display::auth::*;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut auth = XAuth::new()?;

    let entry = auth.entry()?;

    println!("entry: {:?}", entry);

    // maybe we need to interpret the key as hex and then convert it to bytes?
    // looks like we need to interpret it in a very special way according to the qoute below.
    //
    // https://www.x.org/archive/X11R6.8.1/doc/xauth.1.html
    //
    // The data is specified as an even-lengthed string of hexadecimal digits, each pair representing one octet.
    // The first digit of each pair gives the most significant 4 bits of the octet, and the second digit of the pair gives the least significant 4 bits.
    // For example, a 32 character hexkey would represent a 128-bit value.
    // A protocol name consisting of just a single period is treated as an abbreviation for MIT-MAGIC-COOKIE-1.

    /*
    let display = display::open_unix(0, Authenthication::new("MIT-MAGIC-COOKIE-1", "850906d76c379118dc386abadf380652"))?;

    let mut root = display.default_root_window()?;

    println!("root_depth: {}", root.depth());
    */

    /*
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

    window.map()?;

    window.destroy()?;
    */

    loop {}

    Ok(())
}


