use yaxi::clipboard::Clipboard;

use std::time::Duration;
use std::thread;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = Clipboard::new(None)?;

    clipboard.set_text("test 123")?;

    thread::sleep(Duration::from_secs(2));

    Ok(())
}
