use std::any::{Any, TypeId};

///
/// Provides a description for a type that can be used when streaming from a script
///
pub struct ScriptTypeDescription {
    /// The ID of this type so it can be compared to others
    type_id: TypeId
}

impl ScriptTypeDescription {
    ///
    /// Creates a description of the specified type
    ///
    pub fn of<T>() -> ScriptTypeDescription 
    where T: Any {
        ScriptTypeDescription {
            type_id: TypeId::of::<T>()
        }
    }
}

impl PartialEq for ScriptTypeDescription {
    fn eq(&self, compare_to: &ScriptTypeDescription) -> bool {
        self.type_id.eq(&compare_to.type_id)
    }
}
