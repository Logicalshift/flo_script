use std::any::{Any, TypeId};

use gluon_vm::api::{VmType, Pushable, Getable};

///
/// Provides a description for a type that can be used when streaming from a script
///
pub struct ScriptTypeDescription {
    /// The ID of this type so it can be compared to others
    type_id: TypeId
}

impl PartialEq for ScriptTypeDescription {
    fn eq(&self, compare_to: &ScriptTypeDescription) -> bool {
        self.type_id.eq(&compare_to.type_id)
    }
}

///
/// Trait implemented by things that can be used with a script
///
pub trait ScriptType {
    ///
    /// Creates or retrieves a description for this type
    ///
    fn description() -> ScriptTypeDescription;
}

impl<T> ScriptType for T 
where for<'vm, 'value> T: Any+VmType+Getable<'vm, 'value>+Pushable<'vm>+Send {
    fn description() -> ScriptTypeDescription {
        ScriptTypeDescription {
            type_id: TypeId::of::<T>()
        }
    }
}
