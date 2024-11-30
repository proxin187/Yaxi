use std::sync::Arc;
use std::time::Duration;

use crate::display::{self, Atom};
use crate::proto::*;

use atoms::AtomName;
use context::Context;
use event::EventHandler;
use model::{ClipboardData, HandoverStatus};
use {atoms::Atoms, error::Error};

mod atoms;
mod context;
pub mod error;
mod event;
mod model;

pub struct Clipboard {
    context: Context,
    atoms: Atoms,
    handler: Arc<EventHandler>,
}

impl Clipboard {
    pub fn new(display: Option<&str>) -> Result<Clipboard, Error> {
        let display = display::open(display)?;
        let context = Context::from_display(&display)?;
        let atoms = Atoms::new(&display)?;
        let handler = Arc::new(EventHandler::new(context.clone()));

        handler.start()?;

        Ok(Clipboard {
            context,
            atoms,
            handler,
        })
    }

    pub fn clear(&self) -> Result<(), Error> {
        self.context
            .set_selection_owner(self.atoms.selections.clipboard)?;

        self.context.delete_property(self.context.handle.marker())?;

        // TODO: this should clear the clipboard targets but it doesnt
        self.handler.clear(self.atoms.selections.clipboard)?;

        // we need to have this here because self.handler.clear doesnt work
        self.handler.set_targets(self.atoms.selections.clipboard, vec![])?;

        Ok(())
    }

    fn read(&self, target: Atom, selection: Atom) -> Result<Option<ClipboardData>, Error> {
        // 1. try to read from current owner
        if let Some(data) = self.handler.read(selection, target)? {
            return Ok(Some(data));
        }

        // 2. if read failed, try to get from clipboard manager
        if self.context.get_selection_owner_id(self.atoms.selections.clipboard_manager)?.is_none() {
            return Ok(None);
        }

        // 3. request clipboard manager to convert data
        self.context.convert_selection(
            self.atoms.selections.clipboard_manager,
            target,
            self.atoms.protocol.targets,
        )?;

        // 4. wait data
        self.handler.wait_data(target, Duration::from_secs(5))
    }

    fn write(&self, data: Vec<ClipboardData>, selection: Atom) -> Result<(), Error> {
        // 1. check service status
        if self.handler.is_stopped() {
            return Err(Error::ServiceStopped);
        }

        // 2. set owner
        self.context.set_selection_owner(selection)?;

        // 3. prepare targets
        let mut targets = vec![
            self.atoms.protocol.targets,
            self.atoms.protocol.timestamp,
            self.atoms.protocol.multiple,
        ];

        // 4. set data and targets
        let data_clone = data.clone();
        for item in &data_clone {
            targets.push(item.format);
        }

        self.handler.set_targets(selection, targets)?;
        for item in data_clone {
            self.handler.set(selection, item.format, item)?;
        }

        Ok(())
    }
}

impl Clipboard {
    pub fn set_text(&self, text: &str) -> Result<(), Error> {
        let bytes = text.as_bytes();
        let data = vec![
            ClipboardData::from_bytes(bytes.to_vec(), self.atoms.formats.utf8_string),
            ClipboardData::from_bytes(bytes.to_vec(), self.atoms.formats.utf8_mime),
            ClipboardData::from_bytes(bytes.to_vec(), self.atoms.formats.utf8_mime_alt),
        ];

        self.write(data, self.atoms.selections.clipboard)?;
        Ok(())
    }

    pub fn get_text(&self) -> Result<Option<String>, Error> {
        let formats = [
            self.atoms.formats.utf8_string,
            self.atoms.formats.utf8_mime,
            self.atoms.formats.utf8_mime_alt,
        ];

        for format in &formats {
            match self.read(*format, self.atoms.selections.clipboard)? {
                Some(data) => {
                    let bytes = data.bytes().to_owned();
                    let text = String::from_utf8(bytes)?;
                    return Ok(Some(text));
                }
                None => continue,
            }
        }

        Ok(None)
    }

    pub fn get_html(&self) -> Result<Option<(String, Option<String>)>, Error> {
        if let Some(data) = self.read(self.atoms.formats.html, self.atoms.selections.clipboard)? {
            let bytes = data.bytes().to_owned();
            let html = String::from_utf8(bytes)?;
            let alt = self.get_text().ok().flatten();
            return Ok(Some((html, alt)));
        }

        Ok(None)
    }

    pub fn set_html(&self, html: &str, alt: Option<&str>) -> Result<(), Error> {
        let mut data = vec![ClipboardData::from_bytes(
            html.as_bytes().to_vec(),
            self.atoms.formats.html,
        )];

        if let Some(alt) = alt {
            data.push(ClipboardData::from_bytes(
                alt.as_bytes().to_vec(),
                self.atoms.formats.utf8_string,
            ));
        }

        self.write(data, self.atoms.selections.clipboard)?;
        Ok(())
    }

    pub fn get_targets(&self) -> Result<Vec<Atom>, Error> {
        let mut targets = vec![];

        if let Ok(Some(data)) =
            self.read(self.atoms.protocol.targets, self.atoms.selections.clipboard)
        {
            let bytes = data.bytes();
            for i in (0..bytes.len()).step_by(4) {
                let ne_bytes = (&bytes[i..i + 4]).try_into().unwrap();
                let target = Atom::from_ne_bytes(ne_bytes);
                targets.push(target);
            }
        }

        Ok(targets)
    }
}

impl Clipboard {
    fn try_handover_clipboard(&self) -> Result<Option<HandoverStatus>, Error> {
        let selection = self.context.atoms.selections.clipboard;

        log::info!(
            "Handover {} to CLIPBOARD_MANAGER, window: {}",
            selection.display_name(),
            self.context.handle.window_id()
        );

        // 1. check owner
        if !self.context.is_owner(selection)? {
            return Ok(None);
        }

        // 2. check data is not empty
        if self.handler.is_empty(selection)? {
            return Ok(None);
        }

        // 3. make sure the data is not in progress
        std::thread::sleep(Duration::from_millis(50));

        // 4. handover
        self.context.convert_selection_to_self(
            self.context.atoms.selections.clipboard_manager,
            self.context.atoms.protocol.save_targets,
        )?;

        // 5. set in progress
        self.handler.set_in_progress();

        // 6. wait handover changed
        match self.handler.wait_handover_changed() {
            Ok(state) => {
                if state.is_completed() {
                    Ok(Some(state))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        log::trace!("Clipboard dropping, try handover clipboard");
        match self.try_handover_clipboard() {
            Ok(state) => {
                log::trace!("Handover finished, state: {:?}", state);
            }
            Err(e) => {
                log::error!("Handover failed: {}", e);
            }
        }
    }
}
