use super::editor::*;
use super::notebook::*;

///
/// Implementations of this trait host a scripting language used with FlowBetween.
///
pub trait FloScriptHost {
    type Notebook : FloScriptNotebook;
    type Editor : FloScriptEditor;
}
