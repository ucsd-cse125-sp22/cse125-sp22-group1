use rodio::{OutputStream, OutputStreamHandle};

// Audio Context
pub struct AudioCtx {
  pub _stream: OutputStream,
  pub stream_handle: OutputStreamHandle
}

impl AudioCtx {
  pub fn new() -> Self {
    // Get a output stream handle to the default physical sound device
    let output_stream = OutputStream::try_default();
    let (_stream, stream_handle) = match output_stream {
      Ok(s) => s,
      Err(err) => {
        panic!("There was an error in setting up the audio context: {}", err);
      }
    };

    Self {
      _stream,
      stream_handle
    }
  }
}
