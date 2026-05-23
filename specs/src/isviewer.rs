//! The IS-Viewer is a memory-mapped device connected to a PC that allows users to interact with a live program.
//!
//! It also forwards text messages written by programs to a specific address range.
//! Some homebrew test ROMs use this feature to communicate test results on top of the usual display.
//!
//! https://www.behindthecode.ca/n64-is-viewer64/

pub const LENGTH_ADDRESS: u32 = 0x13FF_0014;

pub const BUFFER_START_ADDRESS: u32 = 0x13FF_0020;
pub const BUFFER_END_ADDRESS: u32 = 0x13FF_0220;

pub const BUFFER_SIZE: usize = (BUFFER_END_ADDRESS - BUFFER_START_ADDRESS) as usize;
