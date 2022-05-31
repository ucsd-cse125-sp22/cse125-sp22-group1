use rodio::{OutputStream, OutputStreamHandle};

// Audio Context
pub struct AudioCtx {
    pub _stream: OutputStream,
    pub stream_handle: OutputStreamHandle,
}

impl AudioCtx {
    pub fn new() -> Self {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default()
            .expect("There was an error in setting up the audio context: ");

        Self {
            _stream,
            stream_handle,
        }
    }
}
