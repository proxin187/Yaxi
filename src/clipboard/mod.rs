//! This module provides a clean x11 clipboard interface, notably support for reading and writing
//! clipboard contents.
//!
//! # Basic Usage
//! A example of opening a clipboard connection, note that the argument to `Clipboard::new`
//! specifies the display to open a clipboard connection to similarly to `display::open`.
//!
//! ```no_run
//! use yaxi::clipboard::Clipboard;
//!
//! let clipboard = Clipboard::new(None).unwrap();
//! ```
//!
//! For more examples check out to the individual functions inside the `Clipboard` structure.
//!
//! # Compatability
//! The clipboard module of yaxi is **ONLY** a x11 clipboard implementation and will therefore not
//! work with any non x11 system.
//!

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

/// this structure represents a html selection, notably it contains the raw html and an optional
/// alt attribute which contains the text representation

#[derive(Debug, Clone)]
pub struct Html {
    pub html: String,
    pub alt: Option<String>,
}

impl Html {
    pub fn new(html: String, alt: Option<String>) -> Html {
        Html { html, alt }
    }

    /// this function checks if the html is empty
    pub fn is_empty(&self) -> bool {
        self.html.is_empty()
    }

    /// get a string reference to the html
    pub fn html(&self) -> &str {
        &self.html
    }

    /// get a optional string reference containing the alt
    pub fn alt(&self) -> Option<&str> {
        self.alt.as_deref()
    }
}

/// this enum represents different image formats that can be read and written from the clipboard
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    Bmp,
}

/// this structure represents an image, specificaly it contains the raw bytes and the corresponding
/// image format
#[derive(Debug, Clone)]
pub struct Image {
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
}

impl Image {
    /// get the length of the raw image bytes
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// check if an image selection is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// get the image format
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    /// get a reference to the raw bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// get an owned vector of the raw bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// this structure represents a selection target, eg. STRING_UTF8, text/html and so on.
#[derive(Debug, Clone)]
pub struct Target {
    pub atom: Atom,
    pub name: String,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.atom.id(), self.name)
    }
}

/// this structure represents a single clipboard connection to the x11 server
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
        for item in &data {
            targets.push(item.format);
        }

        self.handler.set_targets(selection, targets)?;
        for item in data {
            self.handler.set(selection, item.format, item)?;
        }

        Ok(())
    }
}

impl Clipboard {
    /// this function clears the clipboard content
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

    /// this sets the clipboard content to a string with the following targets: UTF8_STRING, text/plain;charset=utf-8
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

    /// try to get the clipboard content as text with the following targets: UTF8_STRING, text/plain;charset=utf-8
    pub fn get_text(&self) -> Result<Option<String>, Error> {
        let targets = self.get_targets()?;
        let formats = [
            self.atoms.formats.utf8_string,
            self.atoms.formats.utf8_mime,
            self.atoms.formats.utf8_mime_alt,
        ];

        for format in formats {
            if !targets.contains(&format) {
                continue;
            }
            if let Ok(Some(data)) = self.read(format, self.atoms.selections.clipboard) {
                if data.is_empty() {
                    continue;
                }
                return Ok(Some(String::from_utf8(data.bytes().to_owned())?));
            }
        }

        Ok(None)
    }

    /// try to get the clipboard content as html with text/html and alt with UTF8_STRING, text/plain;charset=utf-8
    pub fn get_html(&self) -> Result<Option<Html>, Error> {
        let targets = self.get_targets()?;
        if !targets.contains(&self.atoms.formats.html) {
            return Ok(None);
        }

        if let Ok(Some(data)) = self.read(self.atoms.formats.html, self.atoms.selections.clipboard)
        {
            if data.is_empty() {
                return Ok(None);
            }
            let html = String::from_utf8(data.bytes().to_owned())?;
            let alt = self.get_text().ok().flatten();
            return Ok(Some(Html { html, alt }));
        }

        Ok(None)
    }

    /// set the clipboard content to html with text/html and optionaly alt with UTF8_STRING, text/plain;charset=utf-8
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

    /// try to get the uri list with text/uri-list
    pub fn get_uri_list(&self) -> Result<Option<Vec<String>>, Error> {
        let targets = self.get_targets()?;
        if !targets.contains(&self.atoms.formats.uri_list) {
            return Ok(None);
        }

        if let Some(data) =
            self.read(self.atoms.formats.uri_list, self.atoms.selections.clipboard)?
        {
            if data.is_empty() {
                return Ok(None);
            }
            let text = String::from_utf8(data.bytes().to_owned())?;
            let list: Vec<_> = text.lines().map(|s| s.to_string()).collect();
            if !list.is_empty() {
                return Ok(Some(list));
            }
        }
        Ok(None)
    }

    /// set the uri list with text/uri-list
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

    /// try to get an image from the clipboard with any of the following formats: image/png, image/jpeg, image/tiff, image/bmp
    pub fn get_image(&self) -> Result<Option<Image>, Error> {
        let targets = self.get_targets()?;
        let formats = [
            self.atoms.formats.png,
            self.atoms.formats.jpeg,
            self.atoms.formats.tiff,
            self.atoms.formats.bmp,
        ];

        for format in formats {
            if !targets.contains(&format) {
                continue;
            }

            if let Ok(Some(data)) = self.read(format, self.atoms.selections.clipboard) {
                if data.is_empty() {
                    continue;
                }
                let format = match format {
                    x if x == self.atoms.formats.png => ImageFormat::Png,
                    x if x == self.atoms.formats.jpeg => ImageFormat::Jpeg,
                    x if x == self.atoms.formats.tiff => ImageFormat::Tiff,
                    x if x == self.atoms.formats.bmp => ImageFormat::Bmp,
                    _ => return Ok(None),
                };
                return Ok(Some(Image {
                    bytes: data.bytes().to_owned(),
                    format,
                }));
            }
        }
        Ok(None)
    }

    /// set the clipboard content to an image with any of the following formats: image/png, image/jpeg, image/tiff, image/bmp
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

    /// get the supported targets for the clibpoard
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

    /// get the supported targets for the clipboard including its name
    pub fn get_targets_with_name(&self) -> Result<Vec<Target>, Error> {
        let mut targets = vec![];

        if let Ok(Some(data)) =
            self.read(self.atoms.protocol.targets, self.atoms.selections.clipboard)
        {
            let bytes = data.bytes();
            for i in (0..bytes.len()).step_by(4) {
                let ne_bytes = (&bytes[i..i + 4]).try_into().unwrap();
                let atom = Atom::from_ne_bytes(ne_bytes);
                let name = atom.name().unwrap_or_else(|| {
                    self.context.get_atom_name(atom).unwrap_or(atom.to_string())
                });
                targets.push(Target { atom, name });
            }
        }

        Ok(targets)
    }
}

impl Clipboard {
    fn try_handover_clipboard(&self) -> Result<Option<HandoverStatus>, Error> {
        let selection = self.context.atoms.selections.clipboard;

        #[cfg(feature = "debug")]
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
        std::thread::sleep(Duration::from_millis(100));

        // 4. handover
        self.context.convert_selection_to_self(
            self.context.atoms.selections.clipboard_manager,
            self.context.atoms.protocol.save_targets,
        )?;

        // 5. set in progress
        self.handler.set_in_progress()?;

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
        #[cfg(feature = "debug")]
        log::trace!("Clipboard dropping, try handover clipboard");

        match self.try_handover_clipboard() {
            Ok(_state) => {
                #[cfg(feature = "debug")]
                log::trace!("Handover finished, state: {:?}", _state);
            }
            Err(_e) => {
                #[cfg(feature = "debug")]
                log::error!("Handover failed: {}", _e);
            }
        }
    }
}
