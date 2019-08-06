use super::super::streams::*;
use super::super::symbol::*;

use gluon::*;

use std::any::*;
use std::collections::HashMap;

///
/// Possible definitions of a symbol in the namespace
///
#[derive(Clone, Debug)]
enum SymbolDefinition {
    Input(InputStreamSource),
}

///
/// Represents a script namespace
///
pub struct GluonScriptNamespace {
    symbols: HashMap<FloScriptSymbol, SymbolDefinition>,

    /// The current thread for generating streaming scripts (or none if it hasn't been created yet)
    streaming: Option<RootedThread>,

    /// The current thread for generating state updating scripts
    state: Option<RootedThread>
}

impl GluonScriptNamespace {
    ///
    /// Creates a new script namespace. The scripting VM is initially not started
    ///
    pub fn new() -> GluonScriptNamespace {
        GluonScriptNamespace {
            symbols:    HashMap::new(),
            streaming:  None,
            state:      None
        }
    }

    ///
    /// Clears this namespace
    ///
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.streaming  = None;
        self.state      = None;
    }

    ///
    /// Defines a particular symbol to be an input stream
    ///
    pub fn define_input_symbol(&mut self, symbol: FloScriptSymbol, input_stream_type: TypeId) {
        let source = InputStreamSource::new(input_stream_type);

        self.symbols.insert(symbol, SymbolDefinition::Input(source));
    }
}
