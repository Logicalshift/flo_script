use super::super::streams::*;
use super::super::symbol::*;
use super::super::error::*;

use gluon::*;
use futures::*;

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

    ///
    /// Creates a stream to read from a particular symbol
    ///
    pub fn read_stream<Symbol: 'static+Clone+Send>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<Stream<Item=Symbol, Error=()>+Send>> {
        use self::SymbolDefinition::*;

        match self.symbols.get(&symbol) {
            None                        => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(Input(input_source))   => Ok(Box::new(input_source.read()?))
        }
    }

    ///
    /// Attaches an input stream to a particular symbol
    ///
    pub fn attach_input<InputStream: Stream<Error=()>>(&self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()> 
    where InputStream::Item: 'static+Clone+Send {
        use self::SymbolDefinition::*;

        match self.symbols.get(&symbol) {
            None                        => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(Input(input_source))   => input_source.attach(input),
            _                           => Err(FloScriptError::NotAnInputSymbol)
        }
    }
}
