use std::result::Result;

///
/// Possible errors from a script call
///
pub enum FloScriptError {
    /// Requested an output or an input with the wrong type
    IncorrectType,

    /// Indicates an error from the script
    ScriptError(String)
}

/// Result from a script operation
type FloScriptResult<T> = Result<T, FloScriptError>;
