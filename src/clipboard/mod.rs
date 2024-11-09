use crate::display::error::Error;
use crate::display::{self, *};
use crate::proto::*;
use crate::window::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

macro_rules! write {
    ($mutex:expr) => {
        $mutex.write().map_err(|_| Error::FailedToLock)
    };
}

macro_rules! read {
    ($mutex:expr) => {
        $mutex.read().map_err(|_| Error::FailedToLock)
    };
}

#[derive(Clone)]
struct ClipboardData {
    bytes: Option<Vec<u8>>,
    format: Atom,
}

impl ClipboardData {
    pub fn new() -> ClipboardData {
        ClipboardData {
            bytes: None,
            format: Atom::new(0),
        }
    }

    #[inline]
    pub fn poll(&self) -> bool {
        self.bytes.is_some()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.bytes = None
    }

    #[inline]
    pub fn get(&self) -> Vec<u8> {
        self.bytes.clone().unwrap_or_default()
    }

    #[inline]
    pub fn set(&mut self, bytes: &[u8], format: Atom) {
        self.bytes.replace(bytes.to_vec());

        self.format = format;
    }
}

#[derive(Clone)]
struct Atoms {
    clipboard: Atom,
    utf8: Atom,
}

#[derive(Clone)]
struct Target {
    window: Window,
    property: Atom,
}

pub struct Clipboard {
    display: Display,
    target: Target,
    atoms: Atoms,
    data: Arc<RwLock<ClipboardData>>,
    listener: ListenerHandle,
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        self.listener.kill();
    }
}

impl Clipboard {
    /// create a new clipboard helper instance
    pub fn new(display: Option<&str>) -> Result<Clipboard, Error> {
        let display = display::open(display)?;
        let root = display.default_root_window()?;

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
            property: display.intern_atom("SKIBIDI_TOILET", false)?,
        };

        let atoms = Atoms {
            clipboard: display.intern_atom("CLIPBOARD", false)?,
            utf8: display.intern_atom("UTF8_STRING", false)?,
        };

        let data = Arc::new(RwLock::new(ClipboardData::new()));

        let listener = ListenerHandle::spawn(
            display.clone(),
            target.clone(),
            atoms.clone(),
            data.clone(),
            Arc::new(AtomicBool::new(false)),
        );

        Ok(Clipboard {
            display,
            target,
            atoms,
            data,
            listener,
        })
    }

    /// set text into the clipboard
    pub fn set_text(&self, text: &str) -> Result<(), Error> {
        self.target
            .window
            .set_selection_owner(self.atoms.clipboard)?;

        write!(self.data).map(|mut lock| lock.set(text.as_bytes(), self.atoms.utf8))
    }

    fn to_string(&self, bytes: Vec<u8>) -> Result<String, Error> {
        String::from_utf8(bytes).map_err(|err| Error::Other { error: err.into() })
    }

    fn get_selection(&self) -> Result<Vec<u8>, Error> {
        write!(self.data)?.reset();

        self.target.window.convert_selection(
            self.atoms.clipboard,
            self.atoms.utf8,
            self.target.property,
        )?;

        while !read!(self.data)?.poll() {}

        read!(self.data).map(|data| data.get())
    }

    // TODO: this deadlocks if the owner terminates during the call

    /// get text from the clipboard
    pub fn get_text(&self) -> Result<String, Error> {
        let owner = self.display.get_selection_owner(self.atoms.clipboard)?;

        match self.display.window_from_id(owner) {
            Ok(window) => {
                let selection = (window.id() != self.target.window.id())
                    .then(|| self.get_selection())
                    .unwrap_or(read!(self.data).map(|data| data.get()))?;

                self.to_string(selection)
            }
            Err(_) => Ok(String::new()),
        }
    }
}

struct ListenerHandle {
    thread: JoinHandle<Result<(), Error>>,
    kill: Arc<AtomicBool>,
}

impl ListenerHandle {
    pub fn spawn(
        display: Display,
        target: Target,
        atoms: Atoms,
        data: Arc<RwLock<ClipboardData>>,
        kill: Arc<AtomicBool>,
    ) -> ListenerHandle {
        let clone = kill.clone();

        ListenerHandle {
            thread: thread::spawn(move || -> Result<(), Error> {
                let mut listener = Listener::new(display, target, atoms, data, clone);

                listener.listen()
            }),
            kill,
        }
    }

    pub fn kill(&mut self) {
        self.kill.store(true, Ordering::Relaxed);

        while !self.thread.is_finished() {}
    }
}

struct Listener {
    display: Display,
    target: Target,
    atoms: Atoms,
    data: Arc<RwLock<ClipboardData>>,
    kill: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(
        display: Display,
        target: Target,
        atoms: Atoms,
        data: Arc<RwLock<ClipboardData>>,
        kill: Arc<AtomicBool>,
    ) -> Listener {
        Listener {
            display,
            target,
            atoms,
            data,
            kill,
        }
    }

    pub fn is_valid(&mut self, target: Atom, property: Atom) -> Result<bool, Error> {
        Ok(target.id() == read!(self.data)?.format.id() && !property.is_null())
    }

    pub fn handle_request(
        &mut self,
        time: u32,
        owner: Window,
        selection: Atom,
        target: Atom,
        property: Atom,
    ) -> Result<(), Error> {
        if self.is_valid(target, property)? {
            let data = read!(self.data)?.get();

            owner.change_property(
                property,
                target,
                PropFormat::Format8,
                PropMode::Replace,
                &data,
            )?;
        }

        owner.send_event(
            Event::SelectionNotify {
                time,
                requestor: owner.id(),
                selection,
                target,
                property: self
                    .is_valid(target, property)?
                    .then(|| property)
                    .unwrap_or(Atom::new(0)),
            },
            vec![],
            true,
        )
    }

    // TODO: the issue may be that this returns a error?

    pub fn listen(&mut self) -> Result<(), Error> {
        while !self.kill.load(Ordering::Relaxed) {
            if self.display.poll_event()? {
                match self.display.next_event()? {
                    Event::SelectionRequest {
                        time,
                        owner,
                        selection,
                        target,
                        property,
                    } => {
                        let owner = self.display.window_from_id(owner)?;

                        self.handle_request(time, owner, selection, target, property)?;
                    }
                    Event::SelectionNotify { property, .. } => {
                        let (bytes, _) = self.target.window.get_property(
                            self.target.property,
                            Atom::ANY_PROPERTY_TYPE,
                            false,
                        )?;

                        let bytes = property
                            .is_null()
                            .then(|| Vec::new())
                            .unwrap_or_else(|| bytes);

                        write!(self.data)?.set(&bytes, self.atoms.utf8);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
