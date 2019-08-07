use super::input_stream_core::*;

use futures::*;
use desync::Desync;

use std::sync::*;

///
/// Reads from an input stream
///
pub struct InputStream<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> {
    /// The ID of this input stream
    stream_id:  usize,

    /// The core where this stream will read from
    core:       Arc<Desync<InputStreamCore<Symbol, Source>>>
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> InputStream<Symbol, Source> {
    ///
    /// Creates a new input stream that will read from the specified core
    /// 
    pub fn new(core: Arc<Desync<InputStreamCore<Symbol, Source>>>) -> InputStream<Symbol, Source> {
        let stream_id = core.sync(|core| core.allocate_stream());

        InputStream {
            stream_id:  stream_id,
            core:       core
        }
    }
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> Drop for InputStream<Symbol, Source> {
    fn drop(&mut self) {
        // Release this stream from the core
        let stream_id = self.stream_id;
        self.core.desync(move |core| core.deallocate_stream(stream_id));
    }
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> Stream for InputStream<Symbol, Source> {
    type Item   = Symbol;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Symbol>, ()> {
        self.core.sync(|core| core.poll_stream(self.stream_id))
    }
}
