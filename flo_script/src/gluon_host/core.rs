use super::editor::*;
use super::core_namespace::*;
use super::super::error::*;
use super::super::symbol::*;
use super::super::editor::*;

use futures::*;

///
/// Core of a script host that targets the Gluon scripting language
/// 
/// See [https://gluon-lang.org] for details on this language.
///
pub struct GluonScriptHostCore {
    /// The root namespace
    root_namespace: GluonScriptNamespace
}

impl GluonScriptHostCore {
    ///
    /// Creates a new script core
    ///
    pub fn new() -> GluonScriptHostCore {
        let root_namespace = GluonScriptNamespace::new();

        GluonScriptHostCore { 
            root_namespace
        }
    }

    ///
    /// Perform an edit on a namespace
    ///
    fn edit_namespace(namespace: &mut GluonScriptNamespace, edit: GluonScriptEdit) {
        use self::GluonScriptEdit::*;
        use self::ScriptEdit::*;

        match edit {
            ScriptEdit(Clear)                                   => { namespace.clear(); }
            ScriptEdit(UndefineSymbol(symbol))                  => { unimplemented!("UndefineSymbol") }
            ScriptEdit(SetInputType(symbol, input_type))        => { namespace.define_input_symbol(symbol, input_type); }
            ScriptEdit(SetStreamingScript(symbol, script_src))  => { unimplemented!("SetStreamingScript") }
            ScriptEdit(SetComputingScript(symbol, script_src))  => { unimplemented!("SetComputingScript") }
            SetRunIo(run_io)                                    => { unimplemented!("SetRunIo") }
            ScriptEdit(WithNamespace(symbol, edits))            => { unimplemented!("WithNamespace") }
        }
    }

    ///
    /// Performs an edit action on this core
    ///
    pub fn edit(&mut self, edit: GluonScriptEdit) {
        Self::edit_namespace(&mut self.root_namespace, edit)
    }

    ///
    /// Creates a stream to read from a particular symbol
    ///
    pub fn read_stream<Symbol: 'static+Clone+Send>(&self, symbol: FloScriptSymbol) -> FloScriptResult<Box<dyn Stream<Item=Symbol, Error=()>+Send>> {
        self.root_namespace.read_stream(symbol)
    }

    ///
    /// Attaches an input stream to a particular symbol
    ///
    pub fn attach_input<InputStream: Stream<Error=()>>(&self, symbol: FloScriptSymbol, input: InputStream) -> FloScriptResult<()> 
    where InputStream::Item: 'static+Clone+Send {
        self.root_namespace.attach_input(symbol, input)
    }
}
