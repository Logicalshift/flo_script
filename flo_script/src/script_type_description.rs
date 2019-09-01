use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::{Debug};
use std::result;

use gluon_vm::api::{VmType, Pushable, Getable};

///
/// Provides a description for a type that can be used when streaming from a script
///
#[derive(Clone)]
pub struct ScriptTypeDescription {
    /// The ID of this type so it can be compared to others
    type_id: TypeId
}

impl ScriptTypeDescription {
    ///
    /// True if this script type matches the specified type
    ///
    pub fn is<T: 'static+ScriptType>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

impl PartialEq for ScriptTypeDescription {
    fn eq(&self, compare_to: &ScriptTypeDescription) -> bool {
        self.type_id.eq(&compare_to.type_id)
    }
}

impl Debug for ScriptTypeDescription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.type_id)
    }
}

///
/// Trait implemented by things that can be used with a script
///
pub trait ScriptType : Any+Clone+Send {
    ///
    /// Creates or retrieves a description for this type
    ///
    fn description() -> ScriptTypeDescription;
}

impl<T> ScriptType for T 
where for<'vm, 'value> T: Any+VmType+Getable<'vm, 'value>+Pushable<'vm>+Clone+Send {
    fn description() -> ScriptTypeDescription {
        ScriptTypeDescription {
            type_id: TypeId::of::<T>()
        }
    }
}
