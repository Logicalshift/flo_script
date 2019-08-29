use gluon::vm::*;
use gluon::vm::api::*;

///
/// Represents a dynamic record
/// 
/// Gluon uses HCons for its own record type, which generates a fixed type. When we import a namespace, we're typically able
/// to judge the types of various parts of the namespace from Gluon but the overall record we need to import does not have
/// a type that's fixed at compile time but at runtime, so we need a way to import a dynamic record.
///
pub struct DynamicRecord<'vm> {
    fields: Vec<(String, Box<dyn FnOnce(&mut ActiveThread<'vm>) -> Result<()>+Send+Sync+'vm>)>
}

impl<'vm> DynamicRecord<'vm> {
    ///
    /// Creates a new dynamic record
    ///
    pub fn new() -> DynamicRecord<'vm> {
        DynamicRecord {
            fields: vec![]
        }
    }

    ///
    /// Adds a field to a dynamic record with a particular name
    ///
    pub fn add_field<Field: 'vm+Pushable<'vm>+Send+Sync>(&mut self, name: String, field: Field) {
        self.fields.push((name, Box::new(|thread| field.push(thread))));
    }
}

impl<'vm> Pushable<'vm> for DynamicRecord<'vm> {
    fn push(self, active_thread: &mut ActiveThread<'vm>) -> Result<()> {
        // Push the field values onto the stack (and keep the field names for later on)
        let mut field_names = vec![];
        for (name, push_field) in self.fields.into_iter() {
            Vec::push(&mut field_names, name);
            push_field(active_thread)?;
        }

        // Fetch the context data
        let thread          = active_thread.thread();
        let context         = active_thread.context();

        // Intern the field names (TODO: error out instead of panicing on a bad name)
        let field_names     = field_names.into_iter()
            .map(|name| thread.global_env().intern(&name).unwrap())
            .collect::<Vec<_>>();

        // Read the fields from the stack to generate the record definition
        let num_fields      = field_names.len();

        // Push the record (this is a bit of a secret squirrel method: it pops 'n' fields from the stack to generate the record)
        context.push_new_record(thread, num_fields, &field_names)?;

        Ok(())
    }
}