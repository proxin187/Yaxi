use yaxi::clipboard::Clipboard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new(None)?;

    clipboard.set_text("test 123")?;

    loop {}

    Ok(())
}
