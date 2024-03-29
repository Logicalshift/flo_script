use super::core_namespace::*;
use super::super::error::*;
use super::super::symbol::*;
use super::super::notebook::*;
use super::super::script_type_description::*;

use gluon::vm::api::*;
use desync::Desync;
use futures::*;

use std::sync::*;

///
/// Provides notebook functionality for a Gluon script host
///
pub struct GluonScriptNotebook {
    /// The namespace that this notebook represents
    namespace: Arc<Desync<GluonScriptNamespace>>
}

impl GluonScriptNotebook {
    ///
    /// Creates a new notebook from a core
    ///
    pub (crate) fn new(namespace: Arc<Desync<GluonScriptNamespace>>) -> GluonScriptNotebook {
        GluonScriptNotebook { namespace }
    }
}

impl FloScriptNotebook for GluonScriptNotebook {
    /// The type of the stream used to receive updates from this notebook
    type UpdateStream = Box<dyn Stream<Item=NotebookUpdate, Error=()>+Send>;

    /// Retrieves a stream of updates for this notebook
    fn updates(&self) -> Self::UpdateStream {
        unimplemented!()
    }

    /// Retrieves a notebook containing the symbols in the specified namespace
    fn namespace(&self, symbol: FloScriptSymbol) -> Option<Self> {
        self.namespace.sync(move |core| {
            core.get_namespace(symbol)
        })
        .map(|namespace| GluonScriptNotebook::new(namespace))
    }

    /// Attaches an input stream to an input symbol. This will replace any existing input stream for that symbol if there is one.
    fn attach_input<InputStream: 'static+Stream<Error=()>+Send>(&self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()> 
    where InputStream::Item: ScriptType {
        self.namespace.sync(move |core| {
            core.attach_input(symbol, input)
        })
    }

    /// Creates an output stream to receive the results from a script associated with the specified symbol
    fn receive_output<OutputItem: 'static+ScriptType>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>>
    where   OutputItem:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
    <OutputItem as VmType>::Type:   Sized {
        self.namespace.sync(move |core| {
            core.read_stream(symbol)
        })
    }

    /// Receives the output stream for the specified symbol as a state stream (which will only return the most recently available symbol when polled)
    fn receive_output_state<OutputItem: 'static+ScriptType>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>>
    where   OutputItem:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
    <OutputItem as VmType>::Type:   Sized {
        self.namespace.sync(move |core| {
            core.read_state_stream(symbol)
        })
    }
}
