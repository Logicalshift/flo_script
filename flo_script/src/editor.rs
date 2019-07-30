use super::symbol::*;

use futures::*;
use std::any::*;

///
/// Represents an edit to a script
///
#[derive(Clone, PartialEq, Debug)]
pub enum ScriptEdit {
    /// Removes all inputs and scripts from the editor
    Clear,

    /// Remove the current definition of a single symbol from the editor
    UndefineSymbol(FloScriptSymbol),

    /// Sets the name of a particular symbol to the specified string
    SetName(FloScriptSymbol, String),

    /// Specifies that a particular symbol is used for input and receives values of the specified type 
    SetInputType(FloScriptSymbol, TypeId),

    /// Specifies that a particular symbol is used as a script, and the contents of the script that it should evaluate
    SetScript(FloScriptSymbol, String),

    /// Performs one or more edits in a namespace (names declared in this namespace are only visible from scripts that are
    /// also in that namespace)
    WithNamespace(FloScriptSymbol, Vec<ScriptEdit>)
}

///
/// The script editor provides a way to change and update a script notebook.
///
pub trait FloScriptEditor : Send+Sync {
    ///
    /// Waits for edits from the specified stream and performs them as they arrive. Returns a future that indicates when the stream
    /// has been consumed.
    /// 
    /// Multiple edits can be sent at once to the script editor if needed: if this occurs, the streams are multiplexed and they are
    /// performed in any order.
    ///
    fn send_edits<Edits: Stream<Item=ScriptEdit, Error=()>>(&self, edits: Edits) -> Box<dyn Future<Item=(), Error=()>>;
}
