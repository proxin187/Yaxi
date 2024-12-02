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

#[derive(Debug, Clone)]
pub struct Html {
    pub html: String,
    pub alt: Option<String>,
}

impl Html {
    pub fn new(html: String, alt: Option<String>) -> Html {
        Html { html, alt }
    }

    pub fn is_empty(&self) -> bool {
        self.html.is_empty()
    }

    pub fn html(&self) -> &str {
        &self.html
    }

    pub fn alt(&self) -> Option<&str> {
        self.alt.as_deref()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    Bmp,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
}

impl Image {
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

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

    fn read(&self, target: Atom, selection: Atom) -> Result<Option<ClipboardData>, Error> {
        // 1. try to read from current owner
        if let Some(data) = self.handler.read(selection, target)? {
            return Ok(Some(data));
        }

        // 2. if read failed, try to get from clipboard manager
        if self
            .context
            .get_selection_owner_id(self.atoms.selections.clipboard_manager)?
            .is_none()
        {
            return Ok(None);
        }

        // 3. request clipboard manager to convert data
        self.context.convert_selection(
            self.atoms.selections.clipboard_manager,
            target,
            self.atoms.protocol.targets,
        )?;

        // 4. wait data
        self.handler.wait_data(target, Duration::from_millis(200))
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
    pub fn clear(&self) -> Result<(), Error> {
        let selection = self.atoms.selections.clipboard;
        self.write(
            vec![ClipboardData::from_bytes(
                vec![],
                self.atoms.formats.utf8_string,
            )],
            selection,
        )?;
        self.handler.clear(selection)?;
        Ok(())
    }

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

        for format in formats {
            match self.read(format, self.atoms.selections.clipboard) {
                Ok(Some(data)) => {
                    if data.is_empty() {
                        continue;
                    }
                    let bytes = data.bytes().to_owned();
                    let text = String::from_utf8(bytes)?;
                    return Ok(Some(text));
                }
                _ => continue,
            }
        }

        Ok(None)
    }

    pub fn get_html(&self) -> Result<Option<Html>, Error> {
        if let Ok(Some(data)) = self.read(self.atoms.formats.html, self.atoms.selections.clipboard)
        {
            if data.is_empty() {
                return Ok(None);
            }
            let bytes = data.bytes().to_owned();
            let html = String::from_utf8(bytes)?;
            let alt = self.get_text().ok().flatten();
            return Ok(Some(Html { html, alt }));
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

    pub fn get_uri_list(&self) -> Result<Option<Vec<String>>, Error> {
        if let Ok(Some(data)) =
            self.read(self.atoms.formats.uri_list, self.atoms.selections.clipboard)
        {
            if data.is_empty() {
                return Ok(None);
            }
            let bytes = data.bytes().to_owned();
            let text = String::from_utf8(bytes)?;
            let list = text.lines().map(|s| s.to_string()).collect::<Vec<_>>();
            if !list.is_empty() {
                return Ok(Some(list));
            }
        }

        Ok(None)
    }

    pub fn set_uri_list(&self, paths: &[&std::path::Path]) -> Result<(), Error> {
        let uris: Vec<String> = paths
            .iter()
            .filter_map(|&path| {
                let path = path.canonicalize().unwrap_or(path.to_path_buf());
                if path.is_absolute() {
                    Some(format!("file://{}", path.display()))
                } else {
                    None
                }
            })
            .collect();

        if uris.is_empty() {
            return Ok(());
        }

        let text = uris.join("\n");
        let data = vec![ClipboardData::from_bytes(
            text.as_bytes().to_vec(),
            self.atoms.formats.uri_list,
        )];

        self.write(data, self.atoms.selections.clipboard)?;
        Ok(())
    }

    pub fn get_image(&self) -> Result<Option<Image>, Error> {
        let formats = [
            self.atoms.formats.png,
            self.atoms.formats.jpeg,
            self.atoms.formats.tiff,
            self.atoms.formats.bmp,
        ];

        for format in formats {
            if let Ok(Some(data)) = self.read(format, self.atoms.selections.clipboard) {
                if data.is_empty() {
                    continue;
                }
                let bytes = data.bytes().to_owned();
                let format = match format {
                    x if x == self.atoms.formats.png => ImageFormat::Png,
                    x if x == self.atoms.formats.jpeg => ImageFormat::Jpeg,
                    x if x == self.atoms.formats.tiff => ImageFormat::Tiff,
                    x if x == self.atoms.formats.bmp => ImageFormat::Bmp,
                    _ => return Ok(None),
                };
                return Ok(Some(Image { bytes, format }));
            }
        }

        Ok(None)
    }

    pub fn set_image(&self, bytes: Vec<u8>, format: ImageFormat) -> Result<(), Error> {
        let format = match format {
            ImageFormat::Png => self.atoms.formats.png,
            ImageFormat::Jpeg => self.atoms.formats.jpeg,
            ImageFormat::Tiff => self.atoms.formats.tiff,
            ImageFormat::Bmp => self.atoms.formats.bmp,
        };

        let data = vec![ClipboardData::from_bytes(bytes, format)];
        self.write(data, self.atoms.selections.clipboard)?;
        Ok(())
    }

    pub fn get_targets(&self) -> Result<Vec<Atom>, Error> {
        let mut targets = vec![];

        // TODO: read results in a Err(Timeout)
        let data = self.read(self.atoms.protocol.targets, self.atoms.selections.clipboard)?;

        if let Some(data) = data {
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
