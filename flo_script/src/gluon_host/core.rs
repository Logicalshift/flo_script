use super::editor::*;
use super::core_namespace::*;

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
    }

    ///
    /// Performs an edit action on this core
    ///
    pub fn edit(&mut self, edit: GluonScriptEdit) {
        Self::edit_namespace(&mut self.root_namespace, edit)
    }
}
