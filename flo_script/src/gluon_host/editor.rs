use super::super::editor::*;

///
/// Actions available for editing/setting up a Gluon script host
///
#[derive(Clone, PartialEq, Debug)]
pub enum GluonScriptEdit {
    /// A standard script editing action
    ScriptEdit(ScriptEdit),

    /// Sets whether or not I/O expressions are evaluated
    SetRunIo(bool)
}
