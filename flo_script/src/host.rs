use super::editor::*;
use super::notebook::*;

///
/// Implementations of this trait host a scripting language used with FlowBetween.
///
pub trait FloScriptHost {
    type Notebook   : FloScriptNotebook;
    type Editor     : FloScriptEditor;

    ///
    /// Retrieves the script notebook for this host
    /// 
    /// The notebook can be used to attach input streams to input symbols and retrieve output streams from scripts.
    /// 
    fn notebook<'a>(&'a self) -> &'a Self::Notebook;

    ///
    /// Retrieves the editor for this host
    /// 
    /// The editor can be used to define input symbols, scripts and namespaces
    /// 
    fn editor<'a>(&'a self) -> &'a Self::Editor;
}
