use crate::display::*;

use std::io::{Read, Write};


#[non_exhaustive]
pub struct MinorOpcodes;

impl MinorOpcodes {
    pub const IS_ACTIVE: u8 = 4;
    pub const QUERY_SCREENS: u8 = 5;
}

// TODO: maybe we should define Xinerama as a struct so that we can store the major_opcode once,
// this can be done in a similar fashion to window

impl<T> Display<T> where T: Send + Sync + Read + Write + TryClone + 'static {
    /*
    pub fn xinerama_major_opcode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    }
    */

    pub fn xinerama_is_active(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        Ok(())
    }
}


