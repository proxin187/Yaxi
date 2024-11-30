use std::collections::{HashMap, VecDeque};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::clipboard::atoms::AtomName;
use crate::display::{Atom, Display};
use crate::proto::Event;
use crate::window::{PropFormat, PropMode, Window};

use super::atoms::Atoms;
use super::context::Context;
use super::error::Error;
use super::model::{Cache, ClipboardData, HandoverState, HandoverStatus};

const MAX_REGULAR_SIZE: usize = 65536;
const INCR_CHUNK_SIZE: usize = 4096;
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Default)]
pub(super) struct EventLoop {
    killed: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<Event>>>,
    condvar: Arc<Condvar>,
    handle: Mutex<Option<JoinHandle<Result<(), Error>>>>,
}

impl EventLoop {
    pub(super) fn start(&self, display: Display) -> Result<(), Error> {
        // TODO: dont use unwrap here

        if self.handle.lock().unwrap().is_some() {
            return Err(Error::EventLoopError("Event loop already running".into()));
        }

        let events = self.events.clone();
        let condvar = self.condvar.clone();
        let killed = self.killed.clone();

        let handle = thread::spawn(move || {
            while !killed.load(Ordering::Relaxed) {
                match display.poll_event() {
                    Ok(true) => {
                        let mut events = events.lock().unwrap();
                        if let Ok(event) = display.next_event() {
                            events.push_back(event);
                            condvar.notify_all();
                        }
                    }
                    Ok(false) => {
                        thread::sleep(POLL_INTERVAL);
                    }
                    Err(e) => {
                        if killed.load(Ordering::Relaxed) {
                            return Ok(());
                        }
                        return Err(Error::EventLoopError(format!("Event source error: {}", e)));
                    }
                }
            }
            Ok(())
        });

        *self.handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    pub(super) fn wait_event(&self) -> Result<Event, Error> {
        if self.killed.load(Ordering::Relaxed) {
            return Err(Error::Terminated);
        }

        let mut events = self.events.lock().unwrap();
        while events.is_empty() && !self.killed.load(Ordering::Relaxed) {
            events = self.condvar.wait(events).unwrap();
        }

        if self.killed.load(Ordering::Relaxed) {
            return Err(Error::Terminated);
        }

        Ok(events.pop_front().unwrap())
    }

    pub(super) fn stop(&self) -> Result<(), Error> {
        self.killed.store(true, Ordering::Relaxed);
        self.condvar.notify_all();

        if let Some(handle) = self.handle.lock().unwrap().take() {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    pub(super) fn is_running(&self) -> bool {
        !self.killed.load(Ordering::Relaxed)
    }

    pub(super) fn clear(&self) -> Result<(), Error> {
        if self.killed.load(Ordering::Relaxed) {
            return Err(Error::Terminated);
        }

        let mut events = self.events.lock().unwrap();
        events.clear();
        Ok(())
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.killed.store(true, Ordering::Relaxed);

        if let Ok(mut handle) = self.handle.lock() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }

        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }
}

#[derive(Default, Clone)]
struct TransferState {
    data: Vec<u8>,
    format: Atom,
    completed: bool,
}

impl TransferState {
    fn empty(format: Atom) -> Self {
        TransferState {
            data: Vec::new(),
            format,
            completed: false,
        }
    }
}

type Transfers = Mutex<HashMap<(Atom, Atom), (TransferState, Arc<(Mutex<bool>, Condvar)>)>>;
type Handover = (Mutex<HandoverStatus>, Condvar);

#[derive(Clone)]
struct State {
    context: Context,
    atoms: Atoms,
    cache: Arc<Cache>,
    transfers: Arc<Transfers>,
    handover: Arc<Handover>,
}

impl State {
    fn new(context: Context) -> Self {
        let atoms = context.atoms;
        State {
            context,
            atoms,
            cache: Arc::new(Cache::new(atoms)),
            transfers: Arc::new(Mutex::new(HashMap::default())),
            handover: Arc::new((Mutex::new(HandoverStatus::default()), Condvar::default())),
        }
    }
}

pub struct EventHandler {
    state: State,
    event_loop: Arc<EventLoop>,
    join_handle: Mutex<Option<JoinHandle<Result<(), Error>>>>,
}

impl EventHandler {
    pub(super) fn new(context: Context) -> Self {
        EventHandler {
            state: State::new(context),
            event_loop: Arc::new(EventLoop::default()),
            join_handle: Mutex::new(None),
        }
    }

    pub(super) fn start(&self) -> Result<(), Error> {
        let event_loop = self.event_loop.clone();
        self.event_loop.start(self.state.context.display.clone())?;

        let state = self.state.clone();
        let handle = thread::spawn(move || {
            while let Ok(event) = event_loop.wait_event() {
                match event {
                    Event::SelectionNotify {
                        time,
                        requestor,
                        selection,
                        target,
                        property,
                    } => {
                        Self::handle_selection_notify(
                            &state, requestor, selection, target, property, time,
                        )?;
                        if selection == state.atoms.selections.clipboard_manager
                            && state.handover.0.lock().unwrap().is_in_progress()
                        {
                            Self::update_handover_status(&state, false, true);
                        }
                    }
                    Event::SelectionRequest {
                        selection,
                        target,
                        property,
                        owner,
                        time,
                    } => {
                        Self::handle_selection_request(
                            &state, selection, target, property, owner, time,
                        )?;
                        if target != state.atoms.protocol.targets
                            && state.handover.0.lock().unwrap().is_in_progress()
                        {
                            Self::update_handover_status(&state, true, false);
                        }
                    }
                    Event::SelectionClear {
                        owner,
                        selection,
                        time,
                    } => {
                        Self::handle_selection_clear(&state, owner, selection, time)?;
                    }
                    _ => {}
                }
            }
            Ok(())
        });

        *self.join_handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    pub(super) fn get(
        &self,
        selection: Atom,
        target: Atom,
    ) -> Result<Option<ClipboardData>, Error> {
        self.state.cache.get(selection, target)
    }

    pub(super) fn is_empty(&self, selection: Atom) -> Result<bool, Error> {
        self.state.cache.is_empty(selection)
    }

    pub(super) fn read(
        &self,
        selection: Atom,
        target: Atom,
    ) -> Result<Option<ClipboardData>, Error> {
        if let Some(owner) = self.state.context.get_selection_owner_id(selection)? {
            if owner == self.state.context.handle.window_id() {
                // Check cache
                if let Some(data) = self.state.cache.get(selection, target)? {
                    return Ok(Some(data));
                }
            }

            // Create transfer state
            let (_state, sync) = {
                let mut transfers = self.state.transfers.lock().unwrap();
                let entry = transfers.entry((selection, target)).or_insert_with(|| {
                    (
                        TransferState::empty(target),
                        Arc::new((Mutex::new(false), Condvar::new())),
                    )
                });
                (entry.0.clone(), entry.1.clone())
            };

            // Start transfer
            self.state
                .context
                .convert_selection_to_self(selection, target)?;

            // TODO: in all of these waiting loops we should maybe consider Condvar with
            // wait_timeout

            // Wait for completion
            let (lock, cvar) = &*sync;
            let mut completed = lock.lock().unwrap();
            let timeout = Instant::now() + TRANSFER_TIMEOUT;

            while !*completed {
                let now = Instant::now();
                if now >= timeout {
                    return Err(Error::Timeout);
                }
                let duration = timeout - now;
                let (new_completed, timeout_result) = cvar.wait_timeout(completed, duration).unwrap();
                completed = new_completed;

                if timeout_result.timed_out() {
                    return Err(Error::Timeout);
                }
            }

            // Get data from cache
            let mut transfers = self.state.transfers.lock().unwrap();

            if let Some((state, _)) = transfers.remove(&(selection, target)) {
                if state.completed {
                    let data = ClipboardData::new(state.data, state.format, 0);
                    // Update cache
                    self.state.cache.set(selection, target, data.clone())?;
                    Ok(Some(data))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn set_targets(&self, selection: Atom, targets: Vec<Atom>) -> Result<(), Error> {
        let bytes = targets
            .iter()
            .flat_map(|&t| t.to_ne_bytes().to_vec())
            .collect::<Vec<_>>();

        let data = ClipboardData::new(bytes, self.state.atoms.protocol.targets, 0);

        self.state
            .cache
            .set(selection, self.state.atoms.protocol.targets, data)
    }

    pub(super) fn set(
        &self,
        selection: Atom,
        target: Atom,
        data: ClipboardData,
    ) -> Result<(), Error> {
        self.state.cache.set(selection, target, data)
    }

    pub(super) fn clear(&self, selection: Atom) -> Result<(), Error> {
        self.state.cache.clear_selection(selection)
    }

    pub(super) fn stop(&self) -> Result<(), Error> {
        self.event_loop.stop()?;
        if let Some(handle) = self.join_handle.lock().unwrap().take() {
            handle.join().unwrap()?;
        }
        Ok(())
    }

    pub(super) fn is_stopped(&self) -> bool {
        !self.event_loop.is_running()
    }
}

impl EventHandler {
    pub fn wait_data(
        &self,
        target: Atom,
        timeout: Duration,
    ) -> Result<Option<ClipboardData>, Error> {
        let transfers = self.state.transfers.clone();
        let selection = self.state.atoms.selections.clipboard;

        // create or get transfer state
        let (_state, sync) = {
            let mut transfers = transfers.lock().unwrap();
            let entry = transfers.entry((selection, target)).or_insert_with(|| {
                (
                    TransferState::default(),
                    Arc::new((Mutex::new(false), Condvar::new())),
                )
            });
            (entry.0.clone(), entry.1.clone())
        };

        // wait for completion
        let (lock, cvar) = &*sync;
        let mut completed = lock.lock().unwrap();
        let deadline = Instant::now() + timeout;

        while !*completed {
            let now = Instant::now();
            if now >= deadline {
                return Err(Error::Timeout);
            }

            let timeout_remaining = deadline - now;
            let (new_completed, timeout_result) = cvar
                .wait_timeout(completed, timeout_remaining)
                .map_err(|_| Error::Timeout)?;
            completed = new_completed;

            if timeout_result.timed_out() {
                return Err(Error::Timeout);
            }
        }

        // get data from cache
        let mut transfers = transfers.lock().unwrap();

        if let Some((state, _)) = transfers.remove(&(selection, target)) {
            if state.completed {
                Ok(Some(ClipboardData::new(state.data, target, 0)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn check_handover_state(&self) -> Result<Option<HandoverStatus>, Error> {
        let (lock, _) = &*self.state.handover;
        let state = lock.lock().unwrap();

        // already completed
        if state.is_completed() {
            return Ok(Some(*state));
        }

        // in progress
        if state.written || state.notified {
            return Ok(Some(*state));
        }

        Ok(None)
    }

    fn update_handover_status(state: &State, written: bool, notified: bool) {
        log::debug!(
            "Updating handover state: written: {}, notified: {}",
            written,
            notified
        );
        let (lock, cvar) = &*state.handover;
        let mut status = lock.lock().unwrap();

        if written {
            status.written = true;
        }
        if notified {
            status.notified = true;
        }

        // if status is completed, notify waiting threads
        if status.written && status.notified {
            status.state = HandoverState::Completed;
            cvar.notify_all();
        }
    }

    pub(super) fn set_in_progress(&self) {
        let (lock, _) = &*self.state.handover;
        let mut status = lock.lock().unwrap();
        status.state = HandoverState::InProgress;
    }

    pub(super) fn wait_handover_changed(&self) -> Result<HandoverStatus, Error> {
        let timeout = Duration::from_millis(500);
        let start = Instant::now();

        while start.elapsed() < timeout {
            if let Some(state) = self.check_handover_state()? {
                if state.written && state.notified {
                    return Ok(state);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        Err(Error::Timeout)
    }

    fn handle_clipboard_manager_notify(
        state: &State,
        _requestor: u32,
        selection: Atom,
        _target: Atom,
        _property: Atom,
    ) -> Result<(), Error> {
        if selection != state.atoms.selections.clipboard_manager {
            return Ok(());
        }

        if state.handover.0.lock().unwrap().is_in_progress() {
            Self::update_handover_status(state, false, true);
        }

        Ok(())
    }
}

impl EventHandler {
    fn handle_selection_notify(
        state: &State,
        requestor: u32,
        selection: Atom,
        target: Atom,
        property: Atom,
        time: u32,
    ) -> Result<(), Error> {
        log::debug!(
            "SelectionNotify: requestor: {} selection: {}, target: {}, property: {}, time: {}",
            requestor,
            selection.display_name(),
            target.display_name(),
            property.display_name(),
            time,
        );

        if selection == state.atoms.selections.clipboard_manager {
            Self::handle_clipboard_manager_notify(state, requestor, selection, target, property)?;
        }

        if property.is_null() {
            // if property is null, it might be a failed request
            return Ok(());
        }

        let mut transfers = state.transfers.lock().unwrap();
        if let Some((ref mut value, sync)) = transfers.get_mut(&(selection, target)) {
            if let Some((data, actual_type)) = state.context.get_property_with_timeout(
                property,
                Atom::default(),
                TRANSFER_TIMEOUT,
            )? {
                if actual_type == state.atoms.protocol.incr {
                    // INCR transfer
                    let size = u32::from_ne_bytes(data[..4].try_into().unwrap());
                    if let Some(data) =
                        state
                            .context
                            .get_property_incr(property, Atom::default(), size)?
                    {
                        value.data = data;
                        value.format = target;
                    }
                } else {
                    // regular transfer
                    value.data.extend_from_slice(&data);
                    value.format = actual_type;
                }
                value.completed = true;
                let (lock, cvar) = &**sync;
                *lock.lock().unwrap() = true;
                cvar.notify_all();
            }
        }

        Ok(())
    }

    fn handle_selection_request(
        state: &State,
        selection: Atom,
        target: Atom,
        property: Atom,
        owner: u32,
        time: u32,
    ) -> Result<(), Error> {
        log::debug!(
            "SelectionRequest: selection: {}, target: {}, property: {}, owner: {}, time: {}",
            selection.display_name(),
            target.display_name(),
            property.display_name(),
            owner,
            time
        );
        let window = state.context.window_from_id(owner)?;

        let success: bool;
        if target == state.atoms.protocol.targets {
            let targets = state.cache.get_targets(selection)?;
            let mut data = Vec::with_capacity(targets.len() * 4);
            for target in targets {
                data.extend_from_slice(&target.to_ne_bytes());
            }

            window.change_property(
                property,
                state.atoms.protocol.atom,
                PropFormat::Format32,
                PropMode::Replace,
                &data,
            )?;

            success = true;
        } else if let Some(data) = state.cache.get(selection, target)? {
            Self::send_data(
                state,
                data,
                window.clone(),
                property,
                selection,
                target,
                time,
            )?;

            success = true;
        } else {
            success = false;
        }

        let property = if success { property } else { Atom::default() };
        let notify = Event::SelectionNotify {
            time,
            requestor: owner,
            selection,
            target,
            property,
        };
        window.send_event(notify, vec![], false)?;

        Ok(())
    }

    fn handle_selection_clear(
        state: &State,
        owner: u32,
        selection: Atom,
        time: u32,
    ) -> Result<(), Error> {
        log::debug!(
            "SelectionClear: owner: {}, selection: {}, time: {}",
            owner,
            selection.display_name(),
            time
        );

        state.cache.clear_selection(selection)?;

        Ok(())
    }

    fn send_data(
        state: &State,
        data: ClipboardData,
        requestor: Window,
        property: Atom,
        selection: Atom,
        target: Atom,
        time: u32,
    ) -> Result<(), Error> {
        let bytes = data.bytes();

        // if property is null, use target
        let actual_property = if property.is_null() { target } else { property };

        if bytes.len() > MAX_REGULAR_SIZE {
            // INCR transfer
            let size = bytes.len() as u32;
            requestor.change_property(
                actual_property,
                state.atoms.protocol.incr,
                PropFormat::Format32,
                PropMode::Replace,
                &size.to_ne_bytes(),
            )?;

            // chunked transfer
            for chunk in bytes.chunks(INCR_CHUNK_SIZE) {
                requestor.change_property(
                    actual_property,
                    data.format,
                    PropFormat::Format8,
                    PropMode::Replace,
                    chunk,
                )?;
                thread::sleep(POLL_INTERVAL);
            }

            // send empty data to indicate completion
            requestor.change_property(
                actual_property,
                data.format,
                PropFormat::Format8,
                PropMode::Replace,
                &[],
            )?;
        } else {
            // regular transfer
            requestor.change_property(
                actual_property,
                data.format,
                PropFormat::Format8,
                PropMode::Replace,
                bytes,
            )?;
        }

        let notify = Event::SelectionNotify {
            time,
            requestor: requestor.id(),
            selection,
            target,
            property: actual_property,
        };

        // send completion notification
        state.context.send_event(notify, vec![], false)?;

        Ok(())
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.event_loop.stop().ok();
        if let Some(handle) = self.join_handle.lock().unwrap().take() {
            handle.join().ok();
        }
    }
}
