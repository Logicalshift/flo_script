use super::error::*;
use super::symbol::*;
use super::script_type_description::*;

use futures::*;
use gluon::vm::api::*;

///
/// Indicates the updates that can occur to a notebook
///
#[derive(Clone, PartialEq, Debug)]
pub enum NotebookUpdate {
    /// A symbol has been specified to define a namespace
    DefinedNamespaceSymbol(FloScriptSymbol),

    /// A symbol has been specified as an input symbol, accepting data of the specified type
    DefinedInputSymbol(FloScriptSymbol, ScriptTypeDescription),

    /// A symbol has been specified as an output symbol, providing data of the specified type
    DefinedOutputSymbol(FloScriptSymbol, ScriptTypeDescription),

    /// An output symbol was added but it could not be defined due to an error
    OutputSymbolError(FloScriptSymbol, FloScriptError),

    /// A series of updates has been performed in a particular namespace
    WithNamespace(FloScriptSymbol, Vec<NotebookUpdate>),

    /// A symbol has been removed from the notebook
    UndefinedSymbol(FloScriptSymbol)
}

///
/// FloScripts are evaluated as 'notebooks'. A notebook is a collection of scripts that provide outputs as
/// streams. Inputs similarly are provided as streams.
///
pub trait FloScriptNotebook : Sized+Send+Sync {
    /// The type of the stream used to receive updates from this notebook
    type UpdateStream  : Stream<Item=NotebookUpdate, Error=()>+Send;

    /// Retrieves a stream of updates for this notebook
    fn updates(&self) -> Self::UpdateStream;

    /// Retrieves a notebook containing the symbols in the specified namespace
    fn namespace(&self, symbol: FloScriptSymbol) -> Option<Self>;

    /// Attaches an input stream to an input symbol. This will replace any existing input stream for that symbol if there is one.
    fn attach_input<InputStream: 'static+Stream<Error=()>+Send>(&self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()>
    where InputStream::Item: ScriptType;

    /// Creates an output stream to receive the results from a script associated with the specified symbol
    /// 
    /// We currently limit ourselves to types that are supported in Gluon; once Rust fully supports specialization, it will be possible to
    /// remove this limit in order to implement the notebook trait on other scripting engines (specialization would make it possible to
    /// return type errors at runtime instead of compile time and avoid restricting the types here).
    fn receive_output<OutputItem: 'static+ScriptType>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>>
    where   OutputItem:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
    <OutputItem as VmType>::Type:   Sized;

    /// Receives the output stream for the specified symbol as a state stream (which will only return the most recently available symbol when polled)
    fn receive_output_state<OutputItem: 'static+ScriptType>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>>
    where   OutputItem:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
    <OutputItem as VmType>::Type:   Sized;
}
