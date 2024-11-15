use crate::display::{Display, Atom};
use crate::display::error::Error;
use crate::window::{Window, PropFormat, PropMode};

use std::collections::HashMap;
use std::string::FromUtf8Error;


/// this represents one of the possible window types defined in ewmh, https://specifications.freedesktop.org/wm-spec/1.3/ar01s05.html#id-1.6.7

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

/// this represents the window geometry width and height, https://specifications.freedesktop.org/wm-spec/1.3/ar01s03.html#id-1.4.6
pub struct DesktopGeometry {
    pub width: u32,
    pub height: u32,
}

/// this represents the window viewport x and y coordinates, https://specifications.freedesktop.org/wm-spec/1.3/ar01s03.html#id-1.4.7
pub struct DesktopViewport {
    pub x: u32,
    pub y: u32,
}

/// Ewmh is a thin wrapper for window allowing the user to implement ewmh compliant applications in
/// a simple maner
pub struct Ewmh {
    pub(crate) display: Display,
    pub(crate) window: Window,
}

impl Ewmh {
    /// get the current active window, (wrapper for _NET_ACTIVE_WINDOW)
    pub fn ewmh_get_active_window(&self) -> Result<Option<u32>, Error> {
        let atom = self.display.intern_atom("_NET_ACTIVE_WINDOW", false)?;

        self.get_u32_property(atom, Atom::WINDOW)
    }

    /// get the client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST has initial mapping order, starting with the oldest window
    pub fn ewmh_get_client_list(&self) -> Result<Option<Vec<u32>>, Error> {
        let atom = self.display.intern_atom("_NET_CLIENT_LIST", false)?;

        self.get_u32_list_property(atom, Atom::WINDOW)
    }

    /// set the client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST has initial mapping order, starting with the oldest window
    pub fn ewmh_set_client_list(&self, clients: &[u32]) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_CLIENT_LIST", false)?;

        self.set_u32_list_property(atom, Atom::WINDOW, PropFormat::Format32, clients)
    }

    /// get the names of virtual desktops, (wrapper for _NET_DESKTOP_NAMES)
    pub fn ewmh_get_desktop_names(&self) -> Result<Option<Vec<Result<String, FromUtf8Error>>>, Error> {
        let atom = self.display.intern_atom("_NET_DESKTOP_NAMES", false)?;
        let utf8 = self.display.intern_atom("UTF8_STRING", false)?;

        self.map_property(atom, utf8, |data, _| {
            data.split(|character| *character == 0)
                .map(|desktop| String::from_utf8(desktop.to_vec()))
                .collect::<Vec<Result<String, FromUtf8Error>>>()
        })
    }

    /// set the names of virtual desktops, (wrapper for _NET_DESKTOP_NAMES)
    pub fn ewmh_set_desktop_names(&self, desktops: &[String]) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_DESKTOP_NAMES", false)?;
        let utf8 = self.display.intern_atom("UTF8_STRING", false)?;

        let bytes = desktops.iter()
            .flat_map(|desktop| [desktop.as_bytes(), &[0]].concat())
            .collect::<Vec<u8>>();

        self.window.change_property(atom, utf8, PropFormat::Format8, PropMode::Replace, &bytes)
    }

    /// get the stacked client list, this list only contains the windows managed by a ewmh compliant window
    /// manager, _NET_CLIENT_LIST_STACKING has bottom-to-top stacking order
    pub fn ewmh_get_client_list_stacking(&self) -> Result<Option<Vec<u32>>, Error> {
        let atom = self.display.intern_atom("_NET_CLIENT_LIST_STACKING", false)?;

        self.get_u32_list_property(atom, Atom::WINDOW)
    }

    /// get the index of the current desktop, (wrapper for _NET_CURRENT_DESKTOP)
    pub fn ewmh_get_current_desktop(&self) -> Result<Option<u32>, Error> {
        let atom = self.display.intern_atom("_NET_CURRENT_DESKTOP", false)?;

        self.get_u32_property(atom, Atom::CARDINAL)
    }

    /// set the index of the current desktop, (wrapper for _NET_CURRENT_DESKTOP)
    pub fn ewmh_set_current_desktop(&self, desktop: u32) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_CURRENT_DESKTOP", false)?;

        self.window.change_property(atom, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &desktop.to_le_bytes())
    }

    /// get the desktop viewport, (wrapper for _NET_DESKTOP_VIEWPORT)
    pub fn ewmh_get_desktop_viewport(&self) -> Result<Option<Vec<DesktopViewport>>, Error> {
        let atom = self.display.intern_atom("_NET_DESKTOP_VIEWPORT", false)?;

        self.map_property(atom, Atom::CARDINAL, |data, _| {
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
        let atom = self.display.intern_atom("_NET_DESKTOP_VIEWPORT", false)?;

        let data = viewport.iter()
            .flat_map(|desktop| [desktop.x.to_le_bytes().to_vec(), desktop.y.to_le_bytes().to_vec()])
            .flatten()
            .collect::<Vec<u8>>();

        self.window.change_property(atom, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &data)
    }

    /// get the desktop geometry, width and height, (wrapper for _NET_DESKTOP_GEOMETRY)
    pub fn ewmh_get_desktop_geometry(&self) -> Result<Option<DesktopGeometry>, Error> {
        let atom = self.display.intern_atom("_NET_DESKTOP_GEOMETRY", false)?;

        let geometry = self.get_u32_list_property(atom, Atom::CARDINAL)?.map(|mut data| {
            data.resize(2, 0);

            DesktopGeometry {
                width: data[0],
                height: data[1],
            }
        });

        Ok(geometry)
    }

    /// The Window Manager MUST set this property on the root window to be the ID of a child window created by himself,
    /// to indicate that a compliant window manager is active, (wrapper for _NET_SUPPORTING_WM_CHECK)
    pub fn ewmh_set_supporting_wm_check(&self, wid: u32) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_SUPPORTING_WM_CHECK", false)?;

        self.window.change_property(atom, Atom::WINDOW, PropFormat::Format32, PropMode::Replace, &wid.to_le_bytes())
    }

    /// The Client SHOULD set this to the title of the window in UTF-8 encoding.
    /// If set, the Window Manager should use this in preference to WM_NAME (wrapper for _NET_WM_NAME)
    pub fn ewmh_set_wm_name(&self, name: &str) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_WM_NAME", false)?;
        let utf8 = self.display.intern_atom("UTF8_STRING", false)?;

        self.window.change_property(atom, utf8, PropFormat::Format8, PropMode::Replace, name.as_bytes())
    }

    /// get the window type, (wrapper for _NET_WM_WINDOW_TYPE)
    pub fn ewmh_get_wm_window_type(&self) -> Result<Vec<EwmhWindowType>, Error> {
        let atom = self.display.intern_atom("_NET_WM_WINDOW_TYPE", false)?;

        let map = HashMap::from([
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_DESKTOP", false)?, EwmhWindowType::Desktop),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_DOCK", false)?, EwmhWindowType::Dock),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_TOOLBAR", false)?, EwmhWindowType::Toolbar),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_MENU", false)?, EwmhWindowType::Menu),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_UTILITY", false)?, EwmhWindowType::Utility),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_SPLASH", false)?, EwmhWindowType::Splash),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_DIALOG", false)?, EwmhWindowType::Dialog),
            (self.display.intern_atom("_NET_WM_WINDOW_TYPE_NORMAL", false)?, EwmhWindowType::Normal),
        ]);

        let type_ = self.get_u32_list_property(atom, Atom::ATOM)?.map(|data| {
            data.iter()
                .filter_map(|atom| map.get(&Atom::new(*atom)).copied())
                .collect::<Vec<EwmhWindowType>>()
        });

        Ok(type_.unwrap_or(Vec::new()))
    }

    /// get the number of desktops, (wrapper for _NET_NUMBER_OF_DESKTOPS)
    pub fn ewmh_get_number_of_desktops(&self) -> Result<Option<u32>, Error> {
        let atom = self.display.intern_atom("_NET_NUMBER_OF_DESKTOPS", false)?;

        self.get_u32_property(atom, Atom::CARDINAL)
    }

    /// set the number of desktops, (wrapper for _NET_NUMBER_OF_DESKTOPS)
    pub fn ewmh_set_number_of_desktops(&self, desktops: u32) -> Result<(), Error> {
        let atom = self.display.intern_atom("_NET_NUMBER_OF_DESKTOPS", false)?;

        self.window.change_property(atom, Atom::CARDINAL, PropFormat::Format32, PropMode::Replace, &desktops.to_le_bytes())
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


