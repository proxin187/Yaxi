use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    let mut clipboard = display.clipboard()?;

    clipboard.set_text("test 123")?;

    loop {}

    Ok(())
}

