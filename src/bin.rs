use yaxi::window::{PropFormat, PropMode, WindowArguments, ValuesBuilder, WindowKind};
use yaxi::proto::{Event, WindowClass, EventMask};
use yaxi::display::{self, Atom};

// NOTE: this is just a test bench for testing features


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open_unix(0)?;

    Ok(())
}


