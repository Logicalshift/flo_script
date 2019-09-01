use super::symbol::*;
use super::script_type_description::*;

use futures::*;
use futures::stream;
use futures::executor;

///
/// Represents an edit to a script
///
#[derive(Clone, PartialEq, Debug)]
pub enum ScriptEdit {
    /// Removes all inputs and scripts from the editor
    Clear,

    /// Remove the current definition of a single symbol from the editor
    UndefineSymbol(FloScriptSymbol),

    /// Specifies that a particular symbol is used for input and receives values of the specified type 
    SetInputType(FloScriptSymbol, ScriptTypeDescription),

    /// Specifies that a particular symbol is used as a script, and the contents of the script that it should evaluate
    /// 
    /// This script will be a streaming script. It receives any inputs as streams and produces its output as a stream.
    /// Streaming scripts will stall when nothing is reading from their output. Only the first script to read from the
    /// output is guaranteed to receive all of the symbols the script produces.
    SetStreamingScript(FloScriptSymbol, String),

    /// Specifies that a particular symbol is used as a script, and the contents of the script that it should evaluate
    /// 
    /// This script will be a 'computing' script. Inputs are treated as streams of states. When this script reads one,
    /// the most recent symbol from the input is passed in. The value will be recomputed, producing a stream of output,
    /// in the event that any of the input values it previously computed change.
    /// 
    /// These scripts essentially act like a cell in a spreadsheet, producing a stream of states from a set of input
    /// values.
    /// 
    /// Nothing will be computed until the first value is 'pulled' from the resulting stream.
    SetComputingScript(FloScriptSymbol, String),

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
    fn send_edits<Edits: 'static+Send+Stream<Item=ScriptEdit, Error=()>>(&self, edits: Edits) -> Box<dyn Future<Item=(), Error=()>>;

    ///
    /// Sends a single edit to a script editor
    ///
    fn edit(&self, edit: ScriptEdit) {
        let edit        = stream::once(Ok(edit));
        let edit_task   = self.send_edits(edit);

        executor::spawn(edit_task).wait_future().unwrap();
    }

    ///
    /// Removes all scripts from the editor
    ///
    fn clear(&self) { self.edit(ScriptEdit::Clear); }

    ///
    /// Marks a symbol that has previously been declared as an input or an output symbol as undefined
    ///
    fn undefine_symbol(&self, symbol: FloScriptSymbol) { self.edit(ScriptEdit::UndefineSymbol(symbol)); }

    ///
    /// Sets the type of the data sent to a particular input symbol
    ///
    fn set_input_type<InputType: ScriptType>(&self, input_symbol: FloScriptSymbol) { self.edit(ScriptEdit::SetInputType(input_symbol, InputType::description())); }

    ///
    /// Defines a streaming script, which will produce an output stream on the specified symbol
    ///
    fn set_streaming_script(&self, output_symbol: FloScriptSymbol, script: &str) { self.edit(ScriptEdit::SetStreamingScript(output_symbol, String::from(script))); }

    ///
    /// Defines a state computing script which will produce an output stream on the specified symbol
    ///
    fn set_computing_script(&self, output_symbol: FloScriptSymbol, script: &str) { self.edit(ScriptEdit::SetComputingScript(output_symbol, String::from(script))); }
}
