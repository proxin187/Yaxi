//! Here you can find implementations of popular x11 extensions

/// xinerama is an extension for having multi-monitor single-screen x11 sessions
pub mod xinerama;

/// an enum for the supported x11 extensions
pub enum Extension {
    Xinerama,
}

impl ToString for Extension {
    fn to_string(&self) -> String {
        match self {
            Extension::Xinerama => String::from("XINERAMA"),
        }
    }
}

impl Extension {
    pub fn len(&self) -> usize {
        self.to_string().len()
    }
}


