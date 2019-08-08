use super::super::streams::*;
use super::super::symbol::*;
use super::super::error::*;

use desync::Desync;
use gluon::*;
use futures::*;

use std::any::*;
use std::sync::*;
use std::collections::HashMap;

///
/// Possible definitions of a symbol in the namespace
///
#[derive(Clone)]
enum SymbolDefinition {
    /// Symbol is an input stream
    Input(InputStreamSource),

    /// Symbol is a namespace
    Namespace(Arc<Desync<GluonScriptNamespace>>)
}

///
/// Represents a script namespace
///
#[derive(Clone)]
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
    pub fn read_stream<Symbol: 'static+Clone+Send>(&mut self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=Symbol, Error=()>+Send>> {
        use self::SymbolDefinition::*;

        match self.symbols.get_mut(&symbol) {
            None                        => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(Input(input_source))   => Ok(Box::new(input_source.read()?)),
            Some(Namespace(_))          => Err(FloScriptError::CannotReadFromANamespace)
        }
    }

    ///
    /// Attaches an input stream to a particular symbol
    ///
    pub fn attach_input<InputStream: 'static+Stream<Error=()>+Send>(&mut self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()> 
    where InputStream::Item: 'static+Clone+Send {
        use self::SymbolDefinition::*;

        match self.symbols.get_mut(&symbol) {
            None                        => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(Input(input_source))   => input_source.attach(input),
            _                           => Err(FloScriptError::NotAnInputSymbol)
        }
    }

    ///
    /// Removes the definition of a symbol from this namespace (if it exists)
    ///
    pub fn undefine_symbol(&mut self, symbol: FloScriptSymbol) {
        self.symbols.remove(&symbol);
    }

    ///
    /// Retrieves a sub-namespace within this namespace. The symbol must already be defined to be a namespace, or must be
    /// undefined (in which case it will be assigned as a namespace)
    ///
    pub fn get_namespace(&mut self, symbol: FloScriptSymbol) -> FloScriptResult<Arc<Desync<GluonScriptNamespace>>> {
        // Insert the namespace if it doesn't already exist
        if self.symbols.get(&symbol).is_none() {
            self.symbols.insert(symbol, SymbolDefinition::Namespace(Arc::new(Desync::new(GluonScriptNamespace::new()))));
        }

        // Retrieve the namespace
        self.symbols.get(&symbol)
            .and_then(|symbol| if let SymbolDefinition::Namespace(symbol) = symbol {
                Some(Arc::clone(symbol))
            } else {
                None
            })
            .ok_or(FloScriptError::NotANamespace)
    }
}
