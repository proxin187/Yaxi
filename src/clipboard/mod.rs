use crate::display::error::Error;
use crate::display::{self, *};
use crate::proto::*;
use crate::window::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::time::Duration;

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Error::FailedToLock)
    };
}

#[derive(Clone)]
pub struct ClipboardData {
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
    fn reset(&mut self) {
        self.bytes = None
    }

    #[inline]
    fn get(&self) -> Vec<u8> {
        self.bytes.clone().unwrap_or_default()
    }

    #[inline]
    fn set(&mut self, bytes: &[u8], format: Atom) {
        self.bytes.replace(bytes.to_vec());

        self.format = format;
    }
}

#[derive(Clone)]
struct Atoms {
    selections: SelectionAtoms,
    manager: ManagerAtoms,
    protocol: ProtocolAtoms,
    formats: FormatAtoms,
}

#[derive(Clone)]
struct SelectionAtoms {
    clipboard: Atom,
    primary: Atom,
    secondary: Atom,
}

#[derive(Clone)]
struct ManagerAtoms {
    manager: Atom,
    save_targets: Atom,
}

#[derive(Clone)]
struct ProtocolAtoms {
    targets: Atom,
    atom: Atom,
    incremental: Atom,
}

#[derive(Clone)]
struct FormatAtoms {
    text: TextFormatAtoms,
    rich: RichFormatAtoms,
}

#[derive(Clone)]
struct TextFormatAtoms {
    utf8_string: Atom,
    utf8_mime: Atom,
    utf8_mime_alt: Atom,
    string: Atom,
    text: Atom,
    plain: Atom,
}

#[derive(Clone)]
struct RichFormatAtoms {
    html: Atom,
    rtf: Atom,
    png: Atom,
    jpeg: Atom,
    tiff: Atom,
    pdf: Atom,
    uri_list: Atom,
}

impl Atoms {
    pub fn new(display: &Display) -> Result<Atoms, Error> {
        Ok(Atoms {
            selections: SelectionAtoms::new(display)?,
            manager: ManagerAtoms::new(display)?,
            protocol: ProtocolAtoms::new(display)?,
            formats: FormatAtoms::new(display)?,
        })
    }
}

impl SelectionAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            clipboard: display.intern_atom("CLIPBOARD", false)?,
            primary: display.intern_atom("PRIMARY", false)?,
            secondary: display.intern_atom("SECONDARY", false)?,
        })
    }
}

impl ManagerAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            manager: display.intern_atom("CLIPBOARD_MANAGER", false)?,
            save_targets: display.intern_atom("SAVE_TARGETS", false)?,
        })
    }
}

impl ProtocolAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            targets: display.intern_atom("TARGETS", false)?,
            atom: display.intern_atom("ATOM", false)?,
            incremental: display.intern_atom("INCR", false)?,
        })
    }
}

impl FormatAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            text: TextFormatAtoms::new(display)?,
            rich: RichFormatAtoms::new(display)?,
        })
    }
}

impl TextFormatAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            utf8_string: display.intern_atom("UTF8_STRING", false)?,
            utf8_mime: display.intern_atom("text/plain;charset=utf-8", false)?,
            utf8_mime_alt: display.intern_atom("text/plain;charset=utf8", false)?,
            string: display.intern_atom("STRING", false)?,
            text: display.intern_atom("TEXT", false)?,
            plain: display.intern_atom("text/plain", false)?,
        })
    }
}

impl RichFormatAtoms {
    fn new(display: &Display) -> Result<Self, Error> {
        Ok(Self {
            html: display.intern_atom("text/html", false)?,
            rtf: display.intern_atom("text/rtf", false)?,
            png: display.intern_atom("image/png", false)?,
            jpeg: display.intern_atom("image/jpeg", false)?,
            tiff: display.intern_atom("image/tiff", false)?,
            pdf: display.intern_atom("application/pdf", false)?,
            uri_list: display.intern_atom("text/uri-list", false)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub atom: Atom,
    pub name: Option<String>,
}

impl From<Atom> for Target {
    fn from(atom: Atom) -> Target {
        Target { atom, name: None }
    }
}

impl Target {
    pub fn new(atom: Atom, name: Option<String>) -> Target {
        Target { atom, name }
    }

    pub fn atom(&self) -> Atom {
        self.atom
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Clone)]
struct Storage {
    window: Window,
    property: Atom,
}

struct Selection {
    data: Mutex<ClipboardData>,
    cond: Condvar,
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            data: Mutex::new(ClipboardData::new()),
            cond: Condvar::new(),
        }
    }

    pub fn reset(&self) -> Result<(), Error> {
        lock!(self.data)?.reset();

        Ok(())
    }

    pub fn set(&self, bytes: &[u8], format: Atom) -> Result<(), Error> {
        lock!(self.data)?.set(bytes, format);

        self.cond.notify_all();

        Ok(())
    }

    pub fn get(&self) -> Result<Vec<u8>, Error> {
        let mut guard = self.data.lock().map_err(|_| Error::FailedToLock)?;

        loop {
            if let Some(bytes) = guard.bytes.clone() {
                return Ok(bytes);
            } else {
                guard = self.cond.wait(guard).map_err(|_| Error::FailedToLock)?;
            }
        }
    }
}

/// this represents the different handover states, Notified means we have recieved a
/// SelectionNotify and Requested means we have recieved a SelectionRequest.
/// the reason why we have these states is because we need to wait until we have been both notified
/// and requested before we are sure the handover was done

#[derive(PartialEq)]
enum HandoverState {
    Idle,
    InProgress,
    Notified,
    Requested,
}

struct Handover {
    state: Mutex<HandoverState>,
    cond: Condvar,
}

impl Handover {
    pub fn new() -> Handover {
        Handover {
            state: Mutex::new(HandoverState::Idle),
            cond: Condvar::new(),
        }
    }
}

pub struct Clipboard {
    display: Display,
    storage: Storage,
    atoms: Atoms,
    selection: Arc<Selection>,
    handover: Arc<Handover>,
    listener: ListenerHandle,
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        if self.is_owner().unwrap_or(false) {
            self.handover().expect("failed to handover to clipboard manager");
        }

        self.listener.kill();
    }
}

impl Clipboard {
    /// create a new clipboard helper instance
    pub fn new(display: Option<&str>) -> Result<Clipboard, Error> {
        let display = display::open(display)?;
        let root = display.default_root_window()?;

        let storage = Storage {
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

        let atoms = Atoms::new(&display)?;
        let selection = Arc::new(Selection::new());
        let handover = Arc::new(Handover::new());

        let listener = ListenerHandle::spawn(
            display.clone(),
            storage.clone(),
            atoms.clone(),
            selection.clone(),
            handover.clone(),
            Arc::new(AtomicBool::new(false)),
        );

        Ok(Clipboard {
            display,
            storage,
            atoms,
            selection,
            handover,
            listener,
        })
    }

    fn handover(&mut self) -> Result<(), Error> {
        let mut state = lock!(self.handover.state)?;

        self.storage.window.convert_selection(
            self.atoms.selections.clipboard,
            self.atoms.manager.save_targets,
            self.storage.property
        )?;

        *state = HandoverState::InProgress;

        drop(self.handover.cond.wait_timeout(state, Duration::from_millis(100)));

        Ok(())
    }

    fn convert_selection(&self, selection: Atom, target: Atom) -> Result<Vec<u8>, Error> {
        self.selection.reset()?;

        self.storage
            .window
            .convert_selection(selection, target, self.storage.property)?;

        self.selection.get()
    }

    fn get_bytes(&self, target: Atom) -> Result<Option<Vec<u8>>, Error> {
        let owner = self
            .display
            .get_selection_owner(self.atoms.selections.clipboard)?;

        let window = self.display.window_from_id(owner)?;
        let selection = if window.id() != self.storage.window.id() {
            self.convert_selection(self.atoms.selections.clipboard, target)?
        } else {
            lock!(self.selection.data).map(|data| data.get())?
        };

        Ok(Some(selection))
    }

    fn get_string(&self, target: Atom) -> Result<Option<String>, Error> {
        let bytes = self.get_bytes(target)?;
        let string = bytes
            .map(|bytes| String::from_utf8(bytes).map_err(|e| Error::Other { error: e.into() }))
            .transpose()?;
        Ok(string)
    }
}

impl Clipboard {
    /// set text into the clipboard
    pub fn set_text(&self, text: &str) -> Result<(), Error> {
        self.storage
            .window
            .set_selection_owner(self.atoms.selections.clipboard)?;

        self.selection.set(text.as_bytes(), self.atoms.formats.text.utf8_string)
    }

    // TODO: this deadlocks if the owner terminates during the call

    /// get text from the clipboard
    pub fn get_text(&self) -> Result<Option<String>, Error> {
        self.get_string(self.atoms.formats.text.utf8_string)
    }

    pub fn get_html(&self) -> Result<Option<String>, Error> {
        self.get_string(self.atoms.formats.rich.html)
    }

    pub fn get_rtf(&self) -> Result<Option<String>, Error> {
        self.get_string(self.atoms.formats.rich.rtf)
    }

    pub fn get_uri_list(&self) -> Result<Option<Vec<String>>, Error> {
        let uris = self
            .get_string(self.atoms.formats.rich.uri_list)?
            .map(|string| string.lines().map(|line| line.to_string()).collect());
        Ok(uris)
    }

    pub fn get_plain_text(&self) -> Result<Option<String>, Error> {
        self.get_string(self.atoms.formats.text.utf8_string)
            .or_else(|_| self.get_string(self.atoms.formats.text.plain))
            .or_else(|_| self.get_string(self.atoms.formats.text.string))
    }

    pub fn get_targets(&self) -> Result<Vec<Target>, Error> {
        let targets =
            self.convert_selection(self.atoms.selections.clipboard, self.atoms.protocol.targets)?;
        let mut atoms = vec![];

        for i in 0..targets.len() / 4 {
            let bytes = &targets[i * 4..(i + 1) * 4];
            if let Ok(atom) = Atom::try_from(bytes) {
                atoms.push(atom);
            }
        }

        let targets = atoms.into_iter().map(Target::from).collect();
        Ok(targets)
    }

    /// this function checks whether we are currently the owner of the selection
    pub fn is_owner(&self) -> Result<bool, Error> {
        self.display.get_selection_owner(self.atoms.selections.clipboard)
            .map(|wid| wid == self.storage.window.id())
    }
}

struct ListenerHandle {
    thread: JoinHandle<Result<(), Error>>,
    kill: Arc<AtomicBool>,
}

impl ListenerHandle {
    pub fn spawn(
        display: Display,
        target: Storage,
        atoms: Atoms,
        selection: Arc<Selection>,
        handover: Arc<Handover>,
        kill: Arc<AtomicBool>,
    ) -> ListenerHandle {
        let clone = kill.clone();

        ListenerHandle {
            thread: thread::spawn(move || -> Result<(), Error> {
                let mut listener = Listener::new(display, target, atoms, selection, handover, clone);

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
    target: Storage,
    atoms: Atoms,
    selection: Arc<Selection>,
    handover: Arc<Handover>,
    kill: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(
        display: Display,
        target: Storage,
        atoms: Atoms,
        selection: Arc<Selection>,
        handover: Arc<Handover>,
        kill: Arc<AtomicBool>,
    ) -> Listener {
        Listener {
            display,
            target,
            atoms,
            selection,
            handover,
            kill,
        }
    }

    pub fn is_valid(&mut self, target: Atom, property: Atom) -> Result<bool, Error> {
        Ok(target.id() == lock!(self.selection.data)?.format.id() && !property.is_null())
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
            let data = lock!(self.selection.data)?.get();

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

    // TODO: this function is quite ugly and could need some cleaning
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

                        if selection == self.atoms.manager.manager {
                            let mut state = lock!(self.handover.state)?;

                            if *state == HandoverState::Notified {
                                self.handover.cond.notify_all();
                            } else if *state == HandoverState::InProgress {
                                *state = HandoverState::Requested;
                            }
                        }
                    }
                    Event::SelectionNotify { property, selection, .. } => {
                        if selection == self.atoms.manager.manager {
                            let mut state = lock!(self.handover.state)?;

                            match *state {
                                HandoverState::InProgress => {
                                    *state = HandoverState::Notified;
                                },
                                HandoverState::Requested => {
                                    self.handover.cond.notify_all();
                                },
                                _ => {},
                            }
                        } else if let Some((bytes, _)) = self.target.window.get_property(
                            self.target.property,
                            Atom::ANY_PROPERTY_TYPE,
                            false,
                        )? {
                            let bytes = property
                                .is_null()
                                .then(|| Vec::new())
                                .unwrap_or_else(|| bytes);

                            self.selection.set(&bytes, self.atoms.formats.text.utf8_string)?;
                        }
                    }
                    Event::SelectionClear { selection, .. } => {
                        if selection == self.atoms.selections.clipboard {
                            self.selection.reset()?;
                        }
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
