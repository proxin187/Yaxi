use xrs::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = display::open_unix(2)?;

    Ok(())
}


