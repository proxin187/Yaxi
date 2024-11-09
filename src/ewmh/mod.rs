use crate::display::{Display, Atom};
use crate::display::error::Error;


#[derive(Clone)]
pub(crate) struct EwmhAtoms {
    pub(crate) net_active_window: Atom,
    pub(crate) net_client_list: Atom,
    pub(crate) net_client_list_stacking: Atom,
    pub(crate) net_current_desktop: Atom,
    pub(crate) net_desktop_geometry: Atom,
    pub(crate) net_desktop_viewport: Atom,
    pub(crate) net_number_of_desktops: Atom,
    pub(crate) net_wm_state: Atom,
    pub(crate) net_showing_desktop: Atom,
    pub(crate) net_wm_allowed_actions: Atom,
    pub(crate) net_wm_desktop: Atom,
    pub(crate) net_wm_name: Atom,
    pub(crate) net_wm_pid: Atom,
    pub(crate) net_wm_visible_name: Atom,
    pub(crate) net_wm_window_type: Atom,
    pub(crate) net_workarea: Atom,
    pub(crate) net_close_window: Atom,
    pub(crate) net_moveresize_window: Atom,
}

impl Display {
    pub(crate) fn get_ewmh_atoms(&self) -> Result<EwmhAtoms, Error> {
        Ok(EwmhAtoms {
            net_active_window: self.intern_atom("_NET_ACTIVE_WINDOW", false)?,
            net_client_list: self.intern_atom("_NET_CLIENT_LIST", false)?,
            net_client_list_stacking: self.intern_atom("_NET_CLIENT_LIST_STACKING", false)?,
            net_current_desktop: self.intern_atom("_NET_CURRENT_DESKTOP", false)?,
            net_desktop_geometry: self.intern_atom("_NET_DESKTOP_GEOMETRY", false)?,
            net_desktop_viewport: self.intern_atom("_NET_DESKTOP_VIEWPORT", false)?,
            net_number_of_desktops: self.intern_atom("_NET_NUMBER_OF_DESKTOPS", false)?,
            net_wm_state: self.intern_atom("_NET_WM_STATE", false)?,
            net_showing_desktop: self.intern_atom("_NET_SHOWING_DESKTOP", false)?,
            net_wm_allowed_actions: self.intern_atom("_NET_WM_ALLOWED_ACTIONS", false)?,
            net_wm_desktop: self.intern_atom("_NET_WM_DESKTOP", false)?,
            net_wm_name: self.intern_atom("_NET_WM_NAME", false)?,
            net_wm_pid: self.intern_atom("_NET_WM_PID", false)?,
            net_wm_visible_name: self.intern_atom("_NET_WM_VISIBLE_NAME", false)?,
            net_wm_window_type: self.intern_atom("_NET_WM_WINDOW_TYPE", false)?,
            net_workarea: self.intern_atom("_NET_WORKAREA", false)?,
            net_close_window: self.intern_atom("_NET_CLOSE_WINDOW", false)?,
            net_moveresize_window: self.intern_atom("_NET_MOVERESIZE_WINDOW", false)?,
        })
    }
}

pub enum EwmhWindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Popup,
    Notification,
    Combo,
    Dnd,
    Normal,
}


