use yaxi::clipboard::Clipboard;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;

    let text = clipboard.get_text()?;

    println!("text: {:?}", text);

    Ok(())
}

