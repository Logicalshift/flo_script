use super::super::streams::*;
use super::super::symbol::*;
use super::super::error::*;

use desync::Desync;
use gluon::*;
use gluon::compiler_pipeline::{CompileValue, Compileable};
use gluon::base::ast::{SpannedExpr};
use gluon::base::symbol::{Symbol};
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

    /// Symbol represents a script that couldn't be compiled
    ScriptError(String),

    /// Compiled computing expression
    Computing(Arc<CompileValue<SpannedExpr<Symbol>>>),

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
    computing: Option<RootedThread>,

    /// Whether or not we'll run I/O operations in this namespace or not
    run_io: bool
}

impl GluonScriptNamespace {
    ///
    /// Creates a new script namespace. The scripting VM is initially not started
    ///
    pub fn new() -> GluonScriptNamespace {
        GluonScriptNamespace {
            symbols:    HashMap::new(),
            streaming:  None,
            computing:  None,
            run_io:     false
        }
    }

    ///
    /// Clears this namespace
    ///
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.streaming  = None;
        self.computing  = None;
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
            None                                => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(ScriptError(description))      => Err(FloScriptError::ScriptError(description.clone())),
            Some(Input(input_source))           => Ok(Box::new(input_source.read_as_stream()?)),
            Some(Computing(expr))               => Err(FloScriptError::Unavailable("computing expressions not implemented yet".to_string())),
            Some(Namespace(_))                  => Err(FloScriptError::CannotReadFromANamespace)
        }
    }

    ///
    /// Creates a stream to read from a particular symbol using the state stream semantics
    ///
    pub fn read_state_stream<Symbol: 'static+Clone+Send>(&mut self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=Symbol, Error=()>+Send>> {
        use self::SymbolDefinition::*;

        match self.symbols.get_mut(&symbol) {
            None                                => Err(FloScriptError::UndefinedSymbol(symbol)),
            Some(ScriptError(description))      => Err(FloScriptError::ScriptError(description.clone())),
            Some(Input(input_source))           => Ok(Box::new(input_source.read_as_state_stream()?)),
            Some(Computing(expr))               => Err(FloScriptError::Unavailable("computing expressions not implemented yet".to_string())),
            Some(Namespace(_))                  => Err(FloScriptError::CannotReadFromANamespace)
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
    pub fn get_or_create_namespace(&mut self, symbol: FloScriptSymbol) -> FloScriptResult<Arc<Desync<GluonScriptNamespace>>> {
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

    ///
    /// Retrieves a sub-namespace, if it is defined
    ///
    pub fn get_namespace(&self, symbol: FloScriptSymbol) -> Option<Arc<Desync<GluonScriptNamespace>>> {
        self.symbols.get(&symbol)
            .and_then(|symbol| if let SymbolDefinition::Namespace(symbol) = symbol {
                Some(Arc::clone(symbol))
            } else {
                None
            })
    }

    ///
    /// Sets whether or not this namespace will run IO commands
    ///
    pub fn set_run_io(&mut self, run_io: bool) {
        // This affects statements compiled after this is set
        self.run_io = run_io;
    }

    ///
    /// Loads a streaming script into this namespace
    ///
    pub fn set_streaming_script(&mut self, symbol: FloScriptSymbol, script: String) {

    }

    ///
    /// Retrieves the computing thread for this namespace, if available
    ///
    fn computing_thread_mut(&mut self) -> &mut RootedThread {
        // Create the thread if it does not already exist
        if self.computing.is_none() {
            self.computing = Some(new_vm());
        }

        // Return the computing thread
        self.computing.as_mut().unwrap()
    }

    ///
    /// Loads a computing script into this namespace
    ///
    pub fn set_computing_script(&mut self, symbol: FloScriptSymbol, script: String) {
        // Attempt to compile the expression
        let computing_thread    = self.computing_thread_mut();
        let mut compiler        = Compiler::new();
        let compiled            = (&script).compile(&mut compiler, &computing_thread, &symbol.name().unwrap_or("".to_string()), &script, None);

        // Report on the result
        match compiled {
            Ok(compiled)        => {
                self.symbols.insert(symbol, SymbolDefinition::Computing(Arc::new(compiled)));
            },
            Err(fail)           => {
                let error_string = fail.emit_string(&compiler.code_map())
                    .unwrap_or("<Error while compiling (could not convert to string)>".to_string());
                self.symbols.insert(symbol, SymbolDefinition::ScriptError(error_string));
            }
        }
    }
}
