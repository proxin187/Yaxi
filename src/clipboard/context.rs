use std::time::{Duration, Instant};

use crate::display::{Atom, Display};
use crate::window::{PropFormat, PropMode, Window};

use super::atoms::Atoms;
use super::error::Error;
use super::{Event, EventMask};

use super::model::AtomHandle;

#[derive(Clone)]
pub struct Context {
    pub(super) display: Display,
    pub(super) handle: AtomHandle,
    pub(super) atoms: Atoms,
}

impl Context {
    pub(super) fn from_display(display: &Display) -> Result<Self, Error> {
        let display = display.clone();
        let handle = AtomHandle::from_display(&display)?;
        let atoms = Atoms::new(&display)?;

        Ok(Context {
            display,
            handle,
            atoms,
        })
    }

    pub(super) fn is_owner(&self, selection: Atom) -> Result<bool, Error> {
        let owner = self.display.get_selection_owner(selection)?;

        Ok(owner == Some(self.handle.window_id()))
    }

    pub(super) fn set_selection_owner(&self, selection: Atom) -> Result<(), Error> {
        self.handle.window().set_selection_owner(selection)?;

        Ok(())
    }

    pub(super) fn get_selection_owner_id(&self, selection: Atom) -> Result<Option<u32>, Error> {
        Ok(self.display.get_selection_owner(selection)?)
    }

    pub(super) fn get_selection_owner(&self, selection: Atom) -> Result<Option<Window>, Error> {
        match self.display.get_selection_owner(selection)? {
            Some(owner) => Ok(Some(self.display.window_from_id(owner)?)),
            None => Ok(None),
        }
    }

    pub(super) fn window_from_id(&self, id: u32) -> Result<Window, Error> {
        Ok(self.display.window_from_id(id)?)
    }

    pub(super) fn delete_property(&self, property: Atom) -> Result<(), Error> {
        self.handle.window().delete_property(property)?;

        Ok(())
    }

    pub(super) fn change_property(
        &self,
        property: Atom,
        type_: Atom,
        format: PropFormat,
        mode: PropMode,
        data: &[u8],
    ) -> Result<(), Error> {
        self.handle
            .window()
            .change_property(property, type_, format, mode, data)?;
        Ok(())
    }

    pub(super) fn convert_selection_to_self(
        &self,
        selection: Atom,
        target: Atom,
    ) -> Result<(), Error> {
        self.handle
            .window()
            .convert_selection(selection, target, self.handle.marker())?;
        Ok(())
    }

    pub(super) fn convert_selection(
        &self,
        selection: Atom,
        target: Atom,
        property: Atom,
    ) -> Result<(), Error> {
        self.handle
            .window()
            .convert_selection(selection, target, property)?;
        Ok(())
    }

    pub(super) fn get_property(
        &self,
        property: Atom,
        type_: Atom,
        delete: bool,
    ) -> Result<Option<(Vec<u8>, Atom)>, Error> {
        let reply = self.handle.window().get_property(property, type_, delete)?;

        if let Some((data, actual_type)) = reply {
            // make sure the data length is valid
            if data.len() % 4 != 0 && actual_type == self.atoms.protocol.atom {
                return Err(Error::InvalidData("Property data length invalid".into()));
            }
            Ok(Some((data, actual_type)))
        } else {
            Ok(None)
        }
    }

    pub(super) fn get_property_incr(
        &self,
        property: Atom,
        type_: Atom,
        size: u32,
    ) -> Result<Option<Vec<u8>>, Error> {
        // first read to get the size
        let initial = self.get_property(property, type_, true)?;

        if let Some((size_data, actual_type)) = initial {
            if actual_type != self.atoms.protocol.incr {
                return Err(Error::InvalidData("Expected INCR property".into()));
            }

            let size = u32::from_ne_bytes(size_data[..4].try_into().unwrap()) as usize;
            let mut buffer = Vec::with_capacity(size);

            // read data chunks
            loop {
                // wait for property change event
                std::thread::sleep(Duration::from_millis(10));

                if let Some((chunk, _)) = self.get_property(property, Atom::default(), true)? {
                    if chunk.is_empty() {
                        break; // done
                    }
                    buffer.extend_from_slice(&chunk);
                }
            }

            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    pub(super) fn get_property_with_timeout(
        &self,
        property: Atom,
        type_: Atom,
        timeout: Duration,
    ) -> Result<Option<(Vec<u8>, Atom)>, Error> {
        let start = Instant::now();

        while start.elapsed() < timeout {
            if let Some(data) = self.get_property(property, type_, false)? {
                return Ok(Some(data));
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Err(Error::Timeout)
    }

    pub(super) fn send_event(
        &self,
        event: Event,
        event_mask: Vec<EventMask>,
        propogate: bool,
    ) -> Result<(), Error> {
        self.handle
            .window()
            .send_event(event, event_mask, propogate)?;
        Ok(())
    }
}
