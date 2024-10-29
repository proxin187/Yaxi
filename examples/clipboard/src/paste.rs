use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    let mut clipboard = display.clipboard()?;

    let text = clipboard.get_text()?;

    println!("text: {:?}", text);

    Ok(())
}

