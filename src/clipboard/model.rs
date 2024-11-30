use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::display::{Atom, Display};
use crate::proto::WindowClass;
use crate::window::{ValuesBuilder, Window, WindowArguments};

use super::atoms::Atoms;
use super::error::Error;

#[derive(Debug, Clone, Copy)]
pub(super) struct TargetSize {
    pub(super) target: Atom,
    pub(super) size: i32,
}

impl TargetSize {
    pub(super) fn new(target: Atom, size: i32) -> TargetSize {
        TargetSize { target, size }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: ClipboardData,
}

pub(super) struct Cache {
    atoms: Atoms,
    data: RwLock<HashMap<(Atom, Atom), CacheEntry>>,
}

impl Cache {
    pub(super) fn new(atoms: Atoms) -> Self {
        let data = RwLock::default();

        Self { atoms, data }
    }

    pub(super) fn get(
        &self,
        selection: Atom,
        target: Atom,
    ) -> Result<Option<ClipboardData>, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;

        let data = guard
            .get(&(selection, target))
            .map(|entry| entry.data.clone());

        Ok(data)
    }

    pub(super) fn get_all(&self, selection: Atom) -> Result<Option<Vec<ClipboardData>>, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;

        let mut data = guard
            .iter()
            .map(|(_, entry)| entry.data.clone())
            .collect::<Vec<_>>();

        data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(Some(data))
    }

    pub(super) fn set(
        &self,
        selection: Atom,
        target: Atom,
        data: ClipboardData,
    ) -> Result<(), Error> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

        guard.insert((selection, target), CacheEntry { data });

        Ok(())
    }

    pub(super) fn set_all(&self, selection: Atom, data: Vec<ClipboardData>) -> Result<(), Error> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

        for entry in data {
            guard.insert((selection, entry.format), CacheEntry { data: entry });
        }

        Ok(())
    }

    pub(super) fn remove(&self, selection: Atom, target: Atom) -> Result<(), Error> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

        guard.remove(&(selection, target));

        Ok(())
    }

    pub(super) fn clear_selection(&self, selection: Atom) -> Result<(), Error> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

        guard.retain(|k, _| k.0 != selection);

        Ok(())
    }

    pub(super) fn clear(&self) -> Result<(), Error> {
        let mut guard = self
            .data
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

        guard.clear();

        Ok(())
    }

    pub(super) fn is_empty(&self, selection: Atom) -> Result<bool, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;
        let is_empty = !guard.iter().any(|(key, _)| key.0 == selection);
        Ok(is_empty)
    }
}

impl Cache {
    pub(super) fn get_targets(&self, selection: Atom) -> Result<Vec<Atom>, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;

        let mut targets = guard
            .iter()
            .filter(|(key, _)| key.0 == selection && !self.atoms.is_side_effect_target(key.1))
            .map(|(key, _)| key.1)
            .collect::<Vec<_>>();

        // Add special targets
        targets.push(self.atoms.protocol.multiple);
        targets.push(self.atoms.protocol.save_targets);
        targets.push(self.atoms.protocol.targets);
        targets.push(self.atoms.protocol.target_sizes);

        Ok(targets)
    }

    pub(crate) fn get_target_size(
        &self,
        selection: Atom,
        target: Atom,
    ) -> Result<Option<TargetSize>, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;
        let size = guard.get(&(selection, target)).map(|entry| {
            let size = if self.atoms.is_side_effect_target(entry.data.format) {
                -1
            } else {
                entry.data.size() as i32
            };
            TargetSize { target, size }
        });
        Ok(size)
    }

    pub(super) fn get_target_sizes(&self, selection: Atom) -> Result<Vec<TargetSize>, Error> {
        let guard = self.data.read().map_err(|e| Error::RwLock(e.to_string()))?;
        let mut sizes = guard
            .iter()
            .filter(|(key, _)| key.0 == selection)
            .map(|(key, entry)| {
                let size = if self.atoms.is_side_effect_target(entry.data.format) {
                    -1
                } else {
                    entry.data.size() as i32
                };
                TargetSize {
                    target: key.1,
                    size,
                }
            })
            .collect::<Vec<_>>();

        // Add special targets
        sizes.push(TargetSize {
            target: self.atoms.protocol.multiple,
            size: 0,
        });
        sizes.push(TargetSize {
            target: self.atoms.protocol.save_targets,
            size: -1,
        });
        sizes.push(TargetSize {
            target: self.atoms.protocol.targets,
            size: sizes.len() as i32 * 4,
        });
        sizes.push(TargetSize {
            target: self.atoms.protocol.target_sizes,
            size: sizes.len() as i32 * 8,
        });

        Ok(sizes)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClipboardData {
    pub(super) bytes: Arc<Vec<u8>>,
    pub(super) format: Atom,
    pub(super) timestamp: u32,
}

impl ClipboardData {
    pub(super) fn new(bytes: Vec<u8>, format: Atom, timestamp: u32) -> ClipboardData {
        ClipboardData {
            bytes: Arc::new(bytes),
            format,
            timestamp,
        }
    }

    pub(super) fn from_bytes(bytes: Vec<u8>, format: Atom) -> ClipboardData {
        ClipboardData {
            bytes: Arc::new(bytes),
            format,
            timestamp: 0,
        }
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn size(&self) -> usize {
        self.bytes.len()
    }
}

#[derive(Clone)]
pub(super) struct AtomHandle {
    window: Window,
    marker: Atom,
}

impl AtomHandle {
    pub(super) fn new(window: Window, marker: Atom) -> AtomHandle {
        AtomHandle { window, marker }
    }

    pub(super) fn window(&self) -> &Window {
        &self.window
    }

    pub(super) fn window_id(&self) -> u32 {
        self.window.id()
    }

    pub(super) fn marker(&self) -> Atom {
        self.marker
    }

    pub(super) fn from_display(display: &Display) -> Result<AtomHandle, Error> {
        let root = display.default_root_window()?;
        let window = root.create_window(WindowArguments {
            depth: root.depth(),
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            class: WindowClass::InputOutput,
            border_width: 0,
            visual: root.visual(),
            values: ValuesBuilder::new(vec![]),
        })?;

        let marker = display.intern_atom("SKIBIDI_TOILET", false)?;
        Ok(AtomHandle::new(window, marker))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum HandoverState {
    #[default]
    Idle = 0,
    InProgress,
    Completed,
}

impl std::fmt::Display for HandoverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandoverState::Idle => write!(f, "Idle"),
            HandoverState::InProgress => write!(f, "InProgress"),
            HandoverState::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct HandoverStatus {
    pub(super) state: HandoverState,
    pub(super) written: bool,
    pub(super) notified: bool,
}

impl std::fmt::Display for HandoverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ state: {}, written: {}, notified: {} }}",
            self.state, self.written, self.notified
        )
    }
}

impl HandoverStatus {
    pub(super) fn is_completed(&self) -> bool {
        self.state == HandoverState::Completed
    }

    pub(super) fn is_in_progress(&self) -> bool {
        self.state == HandoverState::InProgress
    }
}

#[derive(Debug, Default)]
pub(super) struct Handover {
    pub(super) state: Mutex<HandoverStatus>,
    pub(super) condvar: Condvar,
}
