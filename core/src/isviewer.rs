use n64_specs as specs;

use crate::{location::Location, value::Value};

pub type IsViewerControlLocation =
    Location<{ specs::isviewer::LENGTH_ADDRESS }, { specs::isviewer::LENGTH_ADDRESS + 4 }>;

pub type IsViewerBufferLocation =
    Location<{ specs::isviewer::BUFFER_START_ADDRESS }, { specs::isviewer::BUFFER_END_ADDRESS }>;

/// The IS-Viewer is a memory-mapped device connected to a PC that allows users to interact with a live program.
///
/// It also forwards text messages written by programs to a specific address range.
/// Some homebrew test ROMs use this feature to communicate test results on top of the usual display.
///
/// https://www.behindthecode.ca/n64-is-viewer64/
#[derive(Debug)]
pub struct IsViewer {
    /// Input buffer
    buffer: [u8; 0x200],
    buffer_size: usize,

    /// Output text in which buffers are flushed
    text: String,
}

impl Default for IsViewer {
    fn default() -> Self {
        Self {
            buffer: [0; 0x200],
            buffer_size: 0,
            text: String::new(),
        }
    }
}

impl IsViewer {
    pub fn get(&self) -> &str {
        &self.text
    }

    pub(crate) fn push<T: Value>(&mut self, addr: IsViewerBufferLocation, data: T) {
        data.write_mem(&mut self.buffer, addr.relative());

        // TODO wrong? could have written the addr mult times
        self.buffer_size += T::BYTES;
    }

    pub(crate) fn flush(&mut self) {
        let data = String::from_utf8_lossy(&self.buffer[..self.buffer_size]);

        self.text.push_str(&data);
        self.buffer_size = 0;
    }
}
