use super::core::*;
use super::super::symbol::*;
use super::super::notebook::*;

use desync::Desync;
use futures::*;

use std::sync::*;

///
/// Provides notebook functionality for a Gluon script host
///
pub struct GluonScriptNotebook {
    /// The core of the host that this notebook represents
    core: Arc<Desync<GluonScriptHostCore>>
}

impl GluonScriptNotebook {
    ///
    /// Creates a new notebook from a core
    ///
    pub (crate) fn new(core: Arc<Desync<GluonScriptHostCore>>) -> GluonScriptNotebook {
        GluonScriptNotebook { core }
    }
}

impl FloScriptNotebook for GluonScriptNotebook {
    /// The type of the stream used to receive updates from this notebook
    type UpdateStream = Box<Stream<Item=NotebookUpdate, Error=()>+Send>;

    /// Retrieves a stream of updates for this notebook
    fn updates(&self) -> Self::UpdateStream {
        unimplemented!()
    }

    /// Retrieves a notebook containing the symbols in the specified namespace
    fn namespace<'a>(&'a self, symbol: FloScriptSymbol) -> Option<&'a Self> {
        unimplemented!()
    }

    /// Attaches an input stream to an input symbol. This will replace any existing input stream for that symbol if there is one.
    fn attach_input<InputItem: Clone, InputStream: Stream<Item=InputItem, Error=()>>(&self, symbol: FloScriptSymbol, input: InputStream) {
        unimplemented!()
    }

    /// Creates an output stream to receive the results from a script associated with the specified symbol
    fn receive_output<OutputItem>(&self, symbol: FloScriptSymbol) -> Box<dyn Stream<Item=OutputItem, Error=()>> {
        unimplemented!()
    }
}
