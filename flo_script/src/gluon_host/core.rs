use super::editor::*;
use super::core_namespace::*;
use super::super::editor::*;

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
            ScriptEdit(Clear)                                   => { unimplemented!() }
            ScriptEdit(UndefineSymbol(symbol))                  => { unimplemented!() }
            ScriptEdit(SetInputType(symbol, input_type))        => { unimplemented!() }
            ScriptEdit(SetStreamingScript(symbol, script_src))  => { unimplemented!() }
            ScriptEdit(SetComputingScript(symbol, script_src))  => { unimplemented!() }
            SetRunIo(run_io)                                    => { unimplemented!() }
            ScriptEdit(WithNamespace(symbol, edits))            => { unimplemented!() }
        }
    }

    ///
    /// Performs an edit action on this core
    ///
    pub fn edit(&mut self, edit: GluonScriptEdit) {
        Self::edit_namespace(&mut self.root_namespace, edit)
    }
}
