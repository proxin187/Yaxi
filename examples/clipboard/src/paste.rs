use yaxi::window::{Window, WindowArguments, ValuesBuilder};
use yaxi::proto::{WindowClass, Event};
use yaxi::display::{self, Display, Atom};

use std::os::unix::net::UnixStream;


pub struct Atoms {
    selection: Atom,
    utf8: Atom,
}

pub struct Target {
    window: Window<UnixStream>,
    property: Atom,
}

pub struct Getter {
    display: Display<UnixStream>,
    root: Window<UnixStream>,
    target: Target,
    atoms: Atoms,
}

impl Getter {
    pub fn new() -> Result<Getter, Box<dyn std::error::Error>> {
        let mut display = display::open_unix(0)?;

        let mut root = display.default_root_window()?;

        let target = Target {
            window: root.create_window(WindowArguments {
                depth: root.depth(),
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                class: WindowClass::InputOutput,
                border_width: 0,
                visual: root.visual(),
                values: ValuesBuilder::new(vec![]),
            })?,
            property: display.intern_atom("PENGIUN", false)?,
        };

        let atoms = Atoms {
            selection: display.intern_atom("CLIPBOARD", false)?,
            utf8: display.intern_atom("UTF8_STRING", false)?,
        };

        Ok(Getter {
            display,
            root,
            target,
            atoms,
        })
    }

    pub fn read_property(&mut self) {
    }

    pub fn get_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.target.window.convert_selection(self.atoms.selection, self.atoms.utf8, self.target.property)?;

        loop {
            match self.display.next_event()? {
                Event::SelectionNotify { time, requestor, selection, target, property } => {
                },
                _ => {},
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut getter = Getter::new()?;

    getter.get_selection()?;

    Ok(())
}


