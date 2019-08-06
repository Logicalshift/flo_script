use super::error::*;
use super::symbol::*;

use futures::*;

use std::any::*;

///
/// Indicates the updates that can occur to a notebook
///
#[derive(Clone, PartialEq, Debug)]
pub enum NotebookUpdate {
    /// A symbol has been specified to define a namespace
    DefinedNamespaceSymbol(FloScriptSymbol),

    /// A symbol has been specified as an input symbol, accepting data of the specified type
    DefinedInputSymbol(FloScriptSymbol, TypeId),

    /// A symbol has been specified as an output symbol, providing data of the specified type
    DefinedOutputSymbol(FloScriptSymbol, TypeId),

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
pub trait FloScriptNotebook : Send+Sync {
    /// The type of the stream used to receive updates from this notebook
    type UpdateStream  : Stream<Item=NotebookUpdate, Error=()>+Send;

    /// Retrieves a stream of updates for this notebook
    fn updates(&self) -> Self::UpdateStream;

    /// Retrieves a notebook containing the symbols in the specified namespace
    fn namespace<'a>(&'a self, symbol: FloScriptSymbol) -> Option<&'a Self>;

    /// Attaches an input stream to an input symbol. This will replace any existing input stream for that symbol if there is one.
    fn attach_input<InputItem: Clone+Send, InputStream: Stream<Item=InputItem, Error=()>>(&self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()>;

    /// Creates an output stream to receive the results from a script associated with the specified symbol
    fn receive_output<OutputItem: 'static+Clone+Send>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>>;
}
