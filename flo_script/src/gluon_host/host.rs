use super::core::*;
use super::editor::*;
use super::notebook::*;
use super::super::host::*;

use desync::Desync;

use std::sync::*;

///
/// A script host for Gluon scripts
/// 
/// See [https://gluon-lang.org] for details on this language.
///
pub struct GluonScriptHost {
    /// The core is used to execute the scripts asynchronously and process their results 
    core: Arc<Desync<GluonScriptHostCore>>
}

impl GluonScriptHost {
    /// 
    /// Creates a new Gluon script host with no scripts running
    /// 
    pub fn new() -> GluonScriptHost {
        let core = GluonScriptHostCore::new();

        GluonScriptHost {
            core: Arc::new(Desync::new(core))
        }
    }
}

impl FloScriptHost for GluonScriptHost {
    type Notebook   = GluonScriptNotebook;
    type Editor     = GluonScriptEditor;

    ///
    /// Retrieves the script notebook for this host
    /// 
    /// The notebook can be used to attach input streams to input symbols and retrieve output streams from scripts.
    /// 
    fn notebook(&self) -> Self::Notebook {
        let root_namespace = self.core.sync(|core| core.root_namespace());

        GluonScriptNotebook::new(root_namespace)
    }

    ///
    /// Retrieves the editor for this host
    /// 
    /// The editor can be used to define input symbols, scripts and namespaces
    /// 
    fn editor(&self) -> Self::Editor {
        GluonScriptEditor::new(Arc::clone(&self.core))
    }
}
