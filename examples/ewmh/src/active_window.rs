use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = display::open(None)?;
    let root = display.default_root_window()?;

    let active = root.ewmh_get_active_window()?;

    println!("active: {:x?}", active);

    Ok(())
}

