use super::symbol::*;

use std::result::Result;

///
/// Possible errors from a script call
///
#[derive(Clone, PartialEq, Debug)]
pub enum FloScriptError {
    /// The requested feature is not available (with description as to why)
    Unavailable(String),

    /// A requested symbol was not defined
    UndefinedSymbol(FloScriptSymbol),

    /// Tried to attach an input stream to a symbol that is not defined as an input symbol
    NotAnInputSymbol,

    /// Tried to perform an operation on an item that is not a namespace
    NotANamespace,

    /// Attempted to read from a namespace as if it were a stream
    CannotReadFromANamespace,

    /// Requested an output or an input with the wrong type
    IncorrectType,

    /// Indicates an error from the script
    ScriptError(String)
}

/// Result from a script operation
pub type FloScriptResult<T> = Result<T, FloScriptError>;
