use crate::display::error::Error;
use crate::display::*;
use crate::window::*;
use crate::proto::*;

use std::io::{Read, Write};


pub struct Atoms {
    clipboard: Atom,
    utf8: Atom,
}

pub struct Target<T> where T: Send + Sync + Read + Write + TryClone {
    window: Window<T>,
    property: Atom,
}

pub struct Clipboard<T> where T: Send + Sync + Read + Write + TryClone {
    display: Display<T>,
    root: Window<T>,
    target: Target<T>,
    atoms: Atoms,
}

impl<T> Clipboard<T> where T: Send + Sync + Read + Write + TryClone + 'static {
    pub(crate) fn new(mut display: Display<T>) -> Result<Clipboard<T>, Error> {
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
            clipboard: display.intern_atom("CLIPBOARD", false)?,
            utf8: display.intern_atom("UTF8_STRING", false)?,
        };

        Ok(Clipboard {
            display,
            root,
            target,
            atoms,
        })
    }

    /// set text into the clipboard
    pub fn set_text(&mut self, text: &str) -> Result<(), Error> {
        self.target.window.set_selection_owner(self.atoms.clipboard)?;

        // TODO: THIS IS NOT DONE YET
        // We will have to have a seperate thread that listens for SelectionRequests

        Ok(())
    }

    fn read_utf8(&mut self) -> Result<String, Error> {
        let (bytes, _) = self.target.window.get_property(self.target.property, Atom::ANY_PROPERTY_TYPE, false)?;

        String::from_utf8(bytes).map_err(|err| Error::Other { error: err.into() })
    }

    /// get text from the clipboard
    pub fn get_text(&mut self) -> Result<String, Error> {
        self.target.window.convert_selection(self.atoms.clipboard, self.atoms.utf8, self.target.property)?;

        loop {
            match self.display.next_event()? {
                Event::SelectionNotify { property, .. } => {
                    return property.is_null()
                        .then(|| Ok(String::new()))
                        .unwrap_or_else(|| self.read_utf8());
                },
                _ => {},
            }
        }
    }
}


