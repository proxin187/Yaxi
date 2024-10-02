//! This crate provides a high level interface to the x11 [protocol] in pure Rust.
//!
//! [protocol]: https://www.x.org/docs/XProtocol/proto.pdf

/// display is the foundation of an x11 connection
pub mod display;

/// proto contains protocol specific code such as opcodes, replies and so on.
pub mod proto;

/// window contains the core functionality for handling windows
pub mod window;


