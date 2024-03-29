use super::editor::*;
use super::notebook::*;

///
/// Implementations of this trait host a scripting language used with FlowBetween.
///
pub trait FloScriptHost : Send+Sync {
    type Notebook   : FloScriptNotebook;
    type Editor     : FloScriptEditor;

    ///
    /// Retrieves the script notebook for this host
    /// 
    /// The notebook can be used to attach input streams to input symbols and retrieve output streams from scripts.
    /// 
    fn notebook(&self) -> Self::Notebook;

    ///
    /// Retrieves the editor for this host
    /// 
    /// The editor can be used to define input symbols, scripts and namespaces
    /// 
    fn editor(&self) -> Self::Editor;
}
