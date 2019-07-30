use super::symbol::*;

use futures::Stream;
use std::any::*;

///
/// Represents an edit to a script
///
#[derive(Clone, PartialEq, Debug)]
pub enum ScriptEdit {
    /// Removes all inputs and scripts from the editor
    Clear,

    /// Sets the name of a particular symbol to the specified string
    SetName(FloScriptSymbol, String),

    /// Specifies that a particular symbol is used for input and receives values of the specified type 
    SetInputType(FloScriptSymbol, TypeId),

    /// Specifies that a particular symbol is used as a script, and the contents of the script that it should evaluate
    SetScript(FloScriptSymbol, String)
}

///
/// The script editor provides a way to change and update a script notebook.
///
pub trait FloScriptEditor {
    ///
    /// Waits for edits from the specified stream and performs them as they arrive
    ///
    fn receive_edits<Edits: Stream<Item=ScriptEdit, Error=()>>(edits: Edits);
}
