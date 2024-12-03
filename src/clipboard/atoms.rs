use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::display::{Atom, Display};

use super::error::Error;

static NAMES: LazyLock<RwLock<HashMap<Atom, String>>> = LazyLock::new(RwLock::default);

#[derive(Clone, Copy)]
pub(super) struct Atoms {
    pub(super) selections: SelectionAtoms,
    pub(super) protocol: ProtocolAtoms,
    pub(super) formats: FormatAtoms,
}

#[derive(Clone, Copy)]
pub(super) struct SelectionAtoms {
    pub(super) clipboard: Atom,
    pub(super) primary: Atom,
    pub(super) secondary: Atom,
    pub(super) clipboard_manager: Atom,
}

#[derive(Clone, Copy)]
pub(super) struct ProtocolAtoms {
    pub targets: Atom,          // "TARGETS"
    pub multiple: Atom,         // "MULTIPLE"
    pub timestamp: Atom,        // "TIMESTAMP"
    pub target_sizes: Atom,     // "TARGET_SIZES"
    pub save_targets: Atom,     // "SAVE_TARGETS"
    pub delete: Atom,           // "DELETE"
    pub insert_property: Atom,  // "INSERT_PROPERTY"
    pub insert_selection: Atom, // "INSERT_SELECTION"
    pub incr: Atom,             // "INCR"
    pub atom: Atom,             // "ATOM"
    pub none: Atom,             // "NONE"
    pub integer: Atom,          // "INTEGER"
}

#[derive(Clone, Copy)]
pub(super) struct FormatAtoms {
    pub(super) utf8_string: Atom,
    pub(super) utf8_mime: Atom,
    pub(super) utf8_mime_alt: Atom,
    pub(super) string: Atom,
    pub(super) text: Atom,
    pub(super) plain: Atom,
    pub(super) html: Atom,
    pub(super) rtf: Atom,
    pub(super) png: Atom,
    pub(super) jpeg: Atom,
    pub(super) tiff: Atom,
    pub(super) bmp: Atom,
    pub(super) pdf: Atom,
    pub(super) uri_list: Atom,
}

trait InternAtom {
    fn intern(&self, name: &str, only_if_exists: bool) -> Result<Atom, Error>;
}

impl InternAtom for Display {
    fn intern(&self, name: &str, only_if_exists: bool) -> Result<Atom, Error> {
        let atom = self.intern_atom(name, only_if_exists)?;
        let mut guard = NAMES.write().map_err(|e| Error::RwLock(e.to_string()))?;
        guard.insert(atom, name.to_string());
        Ok(atom)
    }
}

pub(super) trait AtomName {
    fn name(&self) -> Option<String>;
    fn display_name(&self) -> String;
}

impl AtomName for Atom {
    fn name(&self) -> Option<String> {
        if let Ok(guard) = NAMES.read() {
            guard.get(self).cloned()
        } else {
            None
        }
    }

    fn display_name(&self) -> String {
        if let Some(name) = self.name() {
            return format!("Atom({}, {})", self.id(), name);
        }
        self.to_string()
    }
}

impl Atoms {
    pub(super) fn new(display: &Display) -> Result<Atoms, Error> {
        Ok(Atoms {
            selections: SelectionAtoms::new(display)?,
            protocol: ProtocolAtoms::new(display)?,
            formats: FormatAtoms::new(display)?,
        })
    }

    pub(super) fn is_side_effect_target(&self, target: Atom) -> bool {
        target == self.protocol.save_targets
            || target == self.protocol.delete
            || target == self.protocol.insert_property
            || target == self.protocol.insert_selection
            || target == self.protocol.multiple
            || target == self.protocol.incr
    }
}

impl SelectionAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            clipboard: display.intern("CLIPBOARD", false)?,
            primary: display.intern("PRIMARY", false)?,
            secondary: display.intern("SECONDARY", false)?,
            clipboard_manager: display.intern("CLIPBOARD_MANAGER", false)?,
        })
    }
}

impl ProtocolAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            targets: display.intern("TARGETS", false)?,
            multiple: display.intern("MULTIPLE", false)?,
            timestamp: display.intern("TIMESTAMP", false)?,
            target_sizes: display.intern("TARGET_SIZES", false)?,
            save_targets: display.intern("SAVE_TARGETS", false)?,
            delete: display.intern("DELETE", false)?,
            insert_property: display.intern("INSERT_PROPERTY", false)?,
            insert_selection: display.intern("INSERT_SELECTION", false)?,
            incr: display.intern("INCR", false)?,
            atom: display.intern("ATOM", false)?,
            none: display.intern("NONE", false)?,
            integer: display.intern("INTEGER", false)?,
        })
    }
}

impl FormatAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            utf8_string: display.intern("UTF8_STRING", false)?,
            utf8_mime: display.intern("text/plain;charset=utf-8", false)?,
            utf8_mime_alt: display.intern("text/plain;charset=utf8", false)?,
            string: display.intern("STRING", false)?,
            text: display.intern("TEXT", false)?,
            plain: display.intern("text/plain", false)?,
            html: display.intern("text/html", false)?,
            rtf: display.intern("text/rtf", false)?,
            png: display.intern("image/png", false)?,
            jpeg: display.intern("image/jpeg", false)?,
            tiff: display.intern("image/tiff", false)?,
            bmp: display.intern("image/bmp", false)?,
            pdf: display.intern("application/pdf", false)?,
            uri_list: display.intern("text/uri-list", false)?,
        })
    }
}
