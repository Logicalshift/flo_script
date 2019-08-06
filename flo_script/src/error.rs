use std::result::Result;

///
/// Possible errors from a script call
///
#[derive(Clone, PartialEq, Debug)]
pub enum FloScriptError {
    /// The requested feature is not availble (with description)
    Unavailable(String),

    /// Requested an output or an input with the wrong type
    IncorrectType,

    /// Indicates an error from the script
    ScriptError(String)
}

/// Result from a script operation
pub type FloScriptResult<T> = Result<T, FloScriptError>;
