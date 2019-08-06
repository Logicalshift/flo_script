use super::input_stream::*;
use super::super::error::*;

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
    /// Creates a new stream reader for this input source
    ///
    pub fn read<SymbolType: 'static+Clone>(&self) -> FloScriptResult<InputStream<SymbolType>> {
        // Can only request the defined type of this input stream
        if TypeId::of::<SymbolType>() != self.input_symbol_type {
            return Err(FloScriptError::IncorrectType)
        }

        Err(FloScriptError::Unavailable("Input stream reading is not implemented yet".to_string()))
    }
}
