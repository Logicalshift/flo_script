use super::input_stream::*;
use super::input_stream_core::*;
use super::super::error::*;

use futures::*;
use desync::Desync;

use std::any::*;
use std::sync::*;

///
/// A structure representing an input stream for a script (provides a possible way to implement a typed input stream for a script host)
///
#[derive(Clone, Debug)]
pub struct InputStreamSource {
    /// The type of symbol that this input stream should return
    input_symbol_type: TypeId,

    /// The stream core object (if it's been attached)
    stream_core: Option<Arc<dyn Any+Send+Sync>>
}

impl InputStreamSource {
    ///
    /// Creates a new input stream that will accept symbols of the specified type
    ///
    pub fn new(input_symbol_type: TypeId) -> InputStreamSource {
        InputStreamSource {
            input_symbol_type:  input_symbol_type,
            stream_core:        None
        }
    }

    ///
    /// Retrieves a reference to the core of this stream source, if available
    ///
    fn core<SymbolType: 'static+Clone+Send>(&mut self) -> FloScriptResult<Arc<Desync<InputStreamCore<SymbolType, Box<dyn Stream<Item=SymbolType, Error=()>+Send>>>>> {
        // Make sure we don't try to create a core of the wrong type
        if TypeId::of::<SymbolType>() != self.input_symbol_type {
            return Err(FloScriptError::IncorrectType)
        }

        // Fetch the stream core
        let stream_core = self.stream_core.get_or_insert_with(|| {
            let new_core = InputStreamCore::<SymbolType, Box<dyn Stream<Item=SymbolType, Error=()>+Send>>::new();

            Arc::new(Desync::new(new_core))
        });

        // Downcast to the correct stream type
        if let Ok(stream_core) = Arc::clone(&stream_core).downcast() {
            Ok(stream_core)
        } else {
            Err(FloScriptError::IncorrectType)
        }
    }

    ///
    /// Sets the stream that's attached to this script input
    ///
    pub fn attach<SymbolStream: 'static+Send+Stream<Error=()>>(&mut self, input_stream: SymbolStream) -> FloScriptResult<()>
    where SymbolStream::Item: 'static+Clone+Send {
        // Replace the stream in the core with the new one that has been passed in
        self.core()?.desync(move |core| { core.replace_stream(Box::new(input_stream)); });

        Ok(())
    } 

    ///
    /// Creates a new stream reader for this input source
    ///
    pub fn read_as_stream<SymbolType: 'static+Clone+Send>(&mut self) -> FloScriptResult<InputStream<SymbolType, Box<dyn Stream<Item=SymbolType, Error=()>+Send>>> {
        // Create a new stream from the core
        let core        = self.core()?;
        let new_stream  = InputStream::new(core);

        Ok(new_stream)
    }
}
