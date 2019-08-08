use super::core_namespace::*;
use super::super::error::*;
use super::super::symbol::*;
use super::super::notebook::*;

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
    where InputStream::Item: Clone+Send {
        self.namespace.sync(move |core| {
            core.attach_input(symbol, input)
        })
    }

    /// Creates an output stream to receive the results from a script associated with the specified symbol
    fn receive_output<OutputItem: 'static+Clone+Send>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=OutputItem, Error=()>+Send>> {
        self.namespace.sync(move |core| {
            core.read_stream(symbol)
        })
    }
}
