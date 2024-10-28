use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    let mut clipboard = display.clipboard()?;

    // TODO: clipboard doesnt work yet.
    //
    // thread '<unnamed>' panicked at /home/proxin/Rust/xrs/src/proto/mod.rs:330:18:
    // internal error: entered unreachable code

    clipboard.set_text("test 123")?;

    loop {}

    Ok(())
}

