use super::input_stream::*;
use super::super::error::*;

use futures::*;

use std::any::*;

///
/// A structure representing an input stream for a script (provides a possible way to implement a typed input stream for a script host)
///
#[derive(Clone, Debug)]
pub struct InputStreamSource {
    /// The type of symbol that this input stream should return
    input_symbol_type: TypeId
}

impl InputStreamSource {
    ///
    /// Creates a new input stream that will accept symbols of the specified type
    ///
    pub fn new(input_symbol_type: TypeId) -> InputStreamSource {
        InputStreamSource {
            input_symbol_type: input_symbol_type
        }
    }

    ///
    /// Sets the stream that's attached to this script input
    ///
    pub fn attach<SymbolStream: Stream<Error=()>>(&self, input_stream: SymbolStream) -> FloScriptResult<()>
    where SymbolStream::Item: 'static+Clone+Send {
        // Can only attach the defined type of this input stream
        if TypeId::of::<SymbolStream::Item>() != self.input_symbol_type {
            return Err(FloScriptError::IncorrectType)
        }

        unimplemented!("Input stream source attach")
    } 

    ///
    /// Creates a new stream reader for this input source
    ///
    pub fn read<SymbolType: 'static+Clone+Send>(&self) -> FloScriptResult<InputStream<SymbolType, Box<dyn Stream<Item=SymbolType, Error=()>+Send>>> {
        // Can only request the defined type of this input stream
        if TypeId::of::<SymbolType>() != self.input_symbol_type {
            return Err(FloScriptError::IncorrectType)
        }

        Err(FloScriptError::Unavailable("Input stream reading is not implemented yet".to_string()))
    }
}
