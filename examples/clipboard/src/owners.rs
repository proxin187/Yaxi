use yaxi::display;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = display::open(None)?;

    let selections = [display.intern_atom("PRIMARY", false)?, display.intern_atom("SECONDARY", false)?, display.intern_atom("CLIPBOARD", false)?];

    for selection in selections {
        let wid = display.get_selection_owner(selection)?;

        println!("[owner] wid: {}", wid);
    }

    Ok(())
}


