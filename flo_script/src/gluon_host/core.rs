use super::editor::*;
use super::core_namespace::*;
use super::super::editor::*;

use desync::Desync;

use std::sync::*;

///
/// Core of a script host that targets the Gluon scripting language
/// 
/// See [https://gluon-lang.org] for details on this language.
///
pub struct GluonScriptHostCore {
    /// The root namespace
    root_namespace: Arc<Desync<GluonScriptNamespace>>,
}

impl GluonScriptHostCore {
    ///
    /// Creates a new script core
    ///
    pub fn new() -> GluonScriptHostCore {
        let root_namespace = GluonScriptNamespace::new();

        GluonScriptHostCore { 
            root_namespace: Arc::new(Desync::new(root_namespace))
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
            ScriptEdit(UndefineSymbol(symbol))                  => { namespace.undefine_symbol(symbol); }
            ScriptEdit(SetInputType(symbol, input_type))        => { namespace.define_input_symbol(symbol, input_type); }
            ScriptEdit(SetStreamingScript(symbol, script_src))  => { namespace.set_streaming_script(symbol, script_src); }
            ScriptEdit(SetComputingScript(symbol, script_src))  => { namespace.set_computing_script(symbol, script_src); }
            SetRunIo(run_io)                                    => { namespace.set_run_io(run_io); }

            ScriptEdit(WithNamespace(symbol, edits))            => {
                namespace.get_or_create_namespace(symbol)
                    .map(|namespace| {
                        namespace.sync(move |namespace| {
                            edits.into_iter().for_each(|edit| {
                                Self::edit_namespace(namespace, ScriptEdit(edit))
                            });
                        });
                    })
                    .ok();
            }
        }
    }

    ///
    /// Retrieves the root namespace for this core
    ///
    pub fn root_namespace(&self) -> Arc<Desync<GluonScriptNamespace>> {
        Arc::clone(&self.root_namespace)
    }

    ///
    /// Performs an edit action on this core
    ///
    pub fn edit(&mut self, edit: GluonScriptEdit) {
        self.root_namespace.sync(|root_namespace| Self::edit_namespace(root_namespace, edit));
    }
}
