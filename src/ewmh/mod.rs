use crate::display::{Display, Atom};
use crate::display::error::Error;
use crate::window::{Window, PropFormat, PropMode};

use std::collections::HashMap;
use std::string::FromUtf8Error;


// TODO: maybe we should load these atoms only when the function is called? or we can have some
// sort of atom cache system


#[derive(Clone, Default)]
pub struct WindowTypeAtoms {
    desktop: Atom,
    dock: Atom,
    toolbar: Atom,
    menu: Atom,
    utility: Atom,
    splash: Atom,
    dialog: Atom,
    normal: Atom,
}

impl WindowTypeAtoms {
    pub fn new(display: &Display) -> Result<WindowTypeAtoms, Error> {
        Ok(WindowTypeAtoms {
            desktop: display.intern_atom("_NET_WM_WINDOW_TYPE_DESKTOP", false)?,
            dock: display.intern_atom("_NET_WM_WINDOW_TYPE_DOCK", false)?,
            toolbar: display.intern_atom("_NET_WM_WINDOW_TYPE_TOOLBAR", false)?,
            menu: display.intern_atom("_NET_WM_WINDOW_TYPE_MENU", false)?,
            utility: display.intern_atom("_NET_WM_WINDOW_TYPE_UTILITY", false)?,
            splash: display.intern_atom("_NET_WM_WINDOW_TYPE_SPLASH", false)?,
            dialog: display.intern_atom("_NET_WM_WINDOW_TYPE_DIALOG", false)?,
            normal: display.intern_atom("_NET_WM_WINDOW_TYPE_NORMAL", false)?,
        })
    }
}

#[derive(Clone, Default)]
pub struct PropertyAtoms {
    active_window: Atom,
    client_list: Atom,
    client_list_stacking: Atom,
    current_desktop: Atom,
    desktop_names: Atom,
    desktop_geometry: Atom,
    desktop_viewport: Atom,
    number_of_desktops: Atom,
    wm_state: Atom,
    showing_desktop: Atom,
    wm_allowed_actions: Atom,
    wm_desktop: Atom,
    wm_name: Atom,
    wm_pid: Atom,
    wm_visible_name: Atom,
    wm_window_type: Atom,
    workarea: Atom,
    close_window: Atom,
    moveresize_window: Atom,
    supporting_wm_check: Atom,
}

impl PropertyAtoms {
    pub fn new(display: &Display) -> Result<PropertyAtoms, Error> {
        Ok(PropertyAtoms {
            active_window: display.intern_atom("_NET_ACTIVE_WINDOW", false)?,
            client_list: display.intern_atom("_NET_CLIENT_LIST", false)?,
            client_list_stacking: display.intern_atom("_NET_CLIENT_LIST_STACKING", false)?,
            current_desktop: display.intern_atom("_NET_CURRENT_DESKTOP", false)?,
            desktop_names: display.intern_atom("_NET_CURRENT_DESKTOP", false)?,
            desktop_geometry: display.intern_atom("_NET_DESKTOP_GEOMETRY", false)?,
            desktop_viewport: display.intern_atom("_NET_DESKTOP_VIEWPORT", false)?,
            number_of_desktops: display.intern_atom("_NET_NUMBER_OF_DESKTOPS", false)?,
            wm_state: display.intern_atom("_NET_WM_STATE", false)?,
            showing_desktop: display.intern_atom("_NET_SHOWING_DESKTOP", false)?,
            wm_allowed_actions: display.intern_atom("_NET_WM_ALLOWED_ACTIONS", false)?,
            wm_desktop: display.intern_atom("_NET_WM_DESKTOP", false)?,
            wm_name: display.intern_atom("_NET_WM_NAME", false)?,
            wm_pid: display.intern_atom("_NET_WM_PID", false)?,
            wm_visible_name: display.intern_atom("_NET_WM_VISIBLE_NAME", false)?,
            wm_window_type: display.intern_atom("_NET_WM_WINDOW_TYPE", false)?,
            workarea: display.intern_atom("_NET_WORKAREA", false)?,
            close_window: display.intern_atom("_NET_CLOSE_WINDOW", false)?,
            moveresize_window: display.intern_atom("_NET_MOVERESIZE_WINDOW", false)?,
            supporting_wm_check: display.intern_atom("_NET_SUPPORTING_WM_CHECK", false)?,
        })
    }
}

#[derive(Clone, Default)]
pub struct Atoms {
    window_type: WindowTypeAtoms,
    property: PropertyAtoms,
    utf8: Atom,
}

impl Atoms {
    pub fn new(display: &Display) -> Result<Atoms, Error> {
        Ok(Atoms {
            window_type: WindowTypeAtoms::new(display)?,
            property: PropertyAtoms::new(display)?,
            utf8: display.intern_atom("UTF8_STRING", false)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EwmhWindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    Normal,
}

pub struct DesktopGeometry {
    pub width: u32,
    pub height: u32,
}

pub struct DesktopViewport {
    pub x: u32,
    pub y: u32,
}

/// Ewmh is a thin wrapper for window allowing the user to implement ewmh compliant applications in
/// a simple maner
pub struct Ewmh {
    pub atoms: Atoms,
    pub window: Window,
}

impl Ewmh {
    /// get the current active window (wrapper for _NET_ACTIVE_WINDOW)
    pub fn ewmh_get_active_window(&self) -> Result<Option<u32>, Error> {
        self.get_u32_property(self.atoms.property.active_window, Atom::WINDOW)
    }

    /// get the client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST has initial mapping order, starting with the oldest window
    pub fn ewmh_get_client_list(&self) -> Result<Option<Vec<u32>>, Error> {
        self.get_u32_list_property(self.atoms.property.client_list, Atom::WINDOW)
    }

    /// set the client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST has initial mapping order, starting with the oldest window
    pub fn ewmh_set_client_list(&self, clients: &[u32]) -> Result<(), Error> {
        self.set_u32_list_property(self.atoms.property.client_list, Atom::WINDOW, PropFormat::Format32, clients)
    }

    /// get the names of virtual desktops, (wrapper for _NET_DESKTOP_NAMES)
    pub fn ewmh_get_desktop_names(&self) -> Result<Option<Vec<Result<String, FromUtf8Error>>>, Error> {
        self.map_property(self.atoms.property.desktop_names, self.atoms.utf8, |data, _| {
            data.split(|character| *character == 0)
                .map(|desktop| String::from_utf8(desktop.to_vec()))
                .collect::<Vec<Result<String, FromUtf8Error>>>()
        })
    }

    /// set the names of virtual desktops, (wrapper for _NET_DESKTOP_NAMES)
    pub fn ewmh_set_desktop_names(&self, desktops: &[String]) -> Result<(), Error> {
        let bytes = desktops.iter()
            .flat_map(|desktop| [desktop.as_bytes(), &[0]].concat())
            .collect::<Vec<u8>>();

        self.window.change_property(self.atoms.property.desktop_names, self.atoms.utf8, PropFormat::Format8, PropMode::Replace, &bytes)
    }

    /// get the stacked client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST_STACKING has bottom-to-top stacking order
    pub fn ewmh_get_client_list_stacking(&self) -> Result<Option<Vec<u32>>, Error> {
        self.get_u32_list_property(self.atoms.property.client_list_stacking, Atom::WINDOW)
    }

    /// get the index of the current desktop, (wrapper for _NET_CURRENT_DESKTOP)
    pub fn ewmh_get_current_desktop(&self) -> Result<Option<u32>, Error> {
        self.get_u32_property(self.atoms.property.current_desktop, Atom::CARDINAL)
    }

    /// set the index of the current desktop, (wrapper for _NET_CURRENT_DESKTOP)
    pub fn ewmh_set_current_desktop(&self, desktop: u32) -> Result<(), Error> {
        self.window.change_property(self.atoms.property.current_desktop, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &desktop.to_le_bytes())
    }

    /// get the desktop viewport, (wrapper for _NET_DESKTOP_VIEWPORT)
    pub fn ewmh_get_desktop_viewport(&self) -> Result<Option<Vec<DesktopViewport>>, Error> {
        self.map_property(self.atoms.property.desktop_viewport, Atom::CARDINAL, |data, _| {
            data.chunks(8)
                .filter(|chunk| chunk.len() == 8)
                .map(|chunk| DesktopViewport {
                    x: u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                    y: u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
                })
                .collect::<Vec<DesktopViewport>>()
        })
    }

    /// set the desktop viewport, (wrapper for _NET_DESKTOP_VIEWPORT)
    pub fn ewmh_set_desktop_viewport(&self, viewport: &[DesktopViewport]) -> Result<(), Error> {
        let data = viewport.iter()
            .flat_map(|desktop| [desktop.x.to_le_bytes().to_vec(), desktop.y.to_le_bytes().to_vec()])
            .flatten()
            .collect::<Vec<u8>>();

        self.window.change_property(self.atoms.property.desktop_viewport, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &data)
    }

    /// get the desktop geometry, width and height, (wrapper for _NET_DESKTOP_GEOMETRY)
    pub fn ewmh_get_desktop_geometry(&self) -> Result<Option<DesktopGeometry>, Error> {
        let geometry = self.get_u32_list_property(self.atoms.property.desktop_geometry, Atom::CARDINAL)?.map(|mut data| {
            data.resize(2, 0);

            DesktopGeometry {
                width: data[0],
                height: data[1],
            }
        });

        Ok(geometry)
    }

    /// The Window Manager MUST set this property on the root window to be the ID of a child window created by himself, to indicate that a compliant window manager is active.
    pub fn ewmh_set_supporting_wm_check(&self, wid: u32) -> Result<(), Error> {
        self.window.change_property(self.atoms.property.supporting_wm_check, Atom::WINDOW, PropFormat::Format32, PropMode::Replace, &wid.to_le_bytes())
    }

    /// The Client SHOULD set this to the title of the window in UTF-8 encoding. If set, the Window Manager should use this in preference to WM_NAME (wrapper for _NET_WM_NAME)
    pub fn ewmh_set_wm_name(&self, name: &str) -> Result<(), Error> {
        self.window.change_property(self.atoms.property.wm_name, self.atoms.utf8, PropFormat::Format8, PropMode::Replace, name.as_bytes())
    }

    /// get the window type, (wrapper for _NET_WM_WINDOW_TYPE)
    pub fn ewmh_get_wm_window_type(&self) -> Result<Vec<EwmhWindowType>, Error> {
        // TODO: using a hashmap and getting the type from there is very clean but quite slow,
        // maybe we should consider more performant alternatives

        let map = HashMap::from([
            (self.atoms.window_type.desktop.id(), EwmhWindowType::Desktop),
            (self.atoms.window_type.dock.id(), EwmhWindowType::Dock),
            (self.atoms.window_type.toolbar.id(), EwmhWindowType::Toolbar),
            (self.atoms.window_type.menu.id(), EwmhWindowType::Menu),
            (self.atoms.window_type.utility.id(), EwmhWindowType::Utility),
            (self.atoms.window_type.splash.id(), EwmhWindowType::Splash),
            (self.atoms.window_type.dialog.id(), EwmhWindowType::Dialog),
            (self.atoms.window_type.normal.id(), EwmhWindowType::Normal),
        ]);

        let type_ = self.get_u32_list_property(self.atoms.property.wm_window_type, Atom::ATOM)?.map(|data| {
            data.iter()
                .filter_map(|atom| map.get(atom).copied())
                .collect::<Vec<EwmhWindowType>>()
        });

        Ok(type_.unwrap_or(Vec::new()))
    }

    /// get the number of desktops, (wrapper for _NET_NUMBER_OF_DESKTOPS)
    pub fn ewmh_get_number_of_desktops(&self) -> Result<Option<u32>, Error> {
        self.get_u32_property(self.atoms.property.number_of_desktops, Atom::CARDINAL)
    }

    /// set the number of desktops, (wrapper for _NET_NUMBER_OF_DESKTOPS)
    pub fn ewmh_set_number_of_desktops(&self, desktops: u32) -> Result<(), Error> {
        self.window.change_property(self.atoms.property.number_of_desktops, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &desktops.to_le_bytes())
    }

    fn get_u32_list_property(&self, property: Atom, type_: Atom) -> Result<Option<Vec<u32>>, Error> {
        self.map_property(property, type_, |data, _| {
            data.chunks(4)
                .filter(|chunk| chunk.len() == 4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect::<Vec<u32>>()
        })
    }

    fn set_u32_list_property(&self, property: Atom, type_: Atom, format: PropFormat, values: &[u32]) -> Result<(), Error> {
        let bytes = values.iter()
            .map(|x| x.to_le_bytes().to_vec())
            .flatten()
            .collect::<Vec<u8>>();

        self.window.change_property(property, type_, format, PropMode::Replace, &bytes)
    }

    fn get_u32_property(&self, property: Atom, type_: Atom) -> Result<Option<u32>, Error> {
        self.map_property(property, type_, |mut data, _| {
            data.resize(4, 0);

            u32::from_le_bytes([data[0], data[1], data[2], data[3]])
        })
    }

    fn map_property<F, R>(&self, property: Atom, type_: Atom, f: F) -> Result<Option<R>, Error> where F: Fn(Vec<u8>, Atom) -> R {
        let wid = self.window.get_property(property, type_, false)?.map(|(data, type_)| f(data, type_));

        Ok(wid)
    }
}


