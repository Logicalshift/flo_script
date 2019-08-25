use super::super::symbol::*;

use gluon::{RootedThread, Thread};
use gluon::vm::{ExternModule, Result, Variants};
use gluon::vm::api::{FunctionRef, ValueRef, ActiveThread, OpaqueValue, Getable, Pushable, UserdataValue};
use gluon::vm::api::generic::{A, B};

use std::collections::{HashSet};

type ValueA = OpaqueValue<RootedThread, A>;
type ValueB = OpaqueValue<RootedThread, B>;

///
/// Data passed through the derived state monad
///
#[derive(Userdata, VmType, Trace, Debug)]
#[gluon_trace(skip)]
#[gluon(vm_type = "flo.computed.prim.DerivedStateData")]
pub struct DerivedStateData {
    // The symbols that the last value of this state depended upon
    dependencies: HashSet<FloScriptSymbol>
}

type ResolveFunction<'vm, TValue> = FunctionRef<'vm, fn(UserdataValue<DerivedStateData>) -> (UserdataValue<DerivedStateData>, TValue)>;

///
/// Monad representing a state value
/// 
/// A state value carries along with the symbols that were read from. We use this later on to decide
/// what to re-evaluate when new data arrives via an input stream.
///
#[derive(VmType)]
pub struct DerivedState<'vm, TValue> {
    // Function for resolving the value of the monad
    resolve: ResolveFunction<'vm, TValue>
}

// Gluon's codegen doesn't seem quite up to the job of dealing with a structure with a function in it, so we need to manually implement getable/pushable
impl<'vm, 'value, TValue> Getable<'vm, 'value> for DerivedState<'vm, TValue> {
    impl_getable_simple!();

    fn from_value(vm: &'vm Thread, value: Variants<'value>) -> Self {
        // Fetch the data from the value
        let data = match value.as_ref() {
            ValueRef::Data(data)    => data,
            other                   => panic!("Unexpected value while retrieving DerivedState: {:?}", other)
        };

        // Read the fields
        let resolve = data.lookup_field(vm, "resolve").expect("Cannot find the `resolve` field while retrieving DerivedState");

        // Decode into a derived state
        DerivedState {
            resolve: ResolveFunction::from_value(vm, resolve)
        }
    }
}

impl<'vm, TValue> Pushable<'vm> for DerivedState<'vm, TValue> {
    fn push(self, context: &mut ActiveThread<'vm>) -> Result<()> {
        unimplemented!()
    }
}

impl<'vm, TValue> DerivedState<'vm, TValue> {
    ///
    /// Creates a new state that is not dependent on any input states
    ///
    pub fn new(value: TValue) -> DerivedState<'vm, TValue> {
        unimplemented!()
    }
}

///
/// Generates the flo.computed.prim extern module for a Gluon VM
///
pub fn load(vm: &Thread) -> Result<ExternModule> {
    vm.register_type::<DerivedStateData>("flo.computed.prim.DerivedStateData", &[])?;
    vm.register_type::<DerivedState<A>>("flo.computed.prim.DerivedState", &["a"])?;

    ExternModule::new(vm, record! {
        type DerivedState a                 => DerivedState<A>,
        type DerivedStateData               => DerivedStateData
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use gluon::*;
    use gluon::import;
    use gluon::vm::api::*;
    use gluon::vm::primitives;

    #[test]
    fn make_type_from_derived_state() {
        let vm = new_vm();
        import::add_extern_module(&vm, "flo.computed.prim", load);

        // Gluon only imports user data types if the corresponding module has previously been imported
        Compiler::default().run_expr::<()>(&vm, "importfs", "import! std.fs\n()").unwrap();
        Compiler::default().run_expr::<()>(&vm, "importcomputed", "import! flo.computed.prim\n()").unwrap();

        // IO monad
        let _some_type = IO::<i32>::make_type(&vm);

        // DirEntry is defined in the standard gluon library: it illustrates this issue does is not a bug with how DerivedState is declared
        let _some_type = UserdataValue::<primitives::DirEntry>::make_type(&vm);
        let _some_type = primitives::DirEntry::make_type(&vm);

        // Ultimate goal of this test: we should be able to get the type for DerivedState
        let _some_type = DerivedState::<i32>::make_type(&vm);
    }

    #[test]
    #[should_panic]
    fn user_data_import_issue_not_fixed() {
        // Without the import! side-effects on the VM, user data types are missing (if this test starts failing, we should be able to remove the import! above and when loading flo.computed)
        let vm = new_vm();
        let _some_type = UserdataValue::<primitives::DirEntry>::make_type(&vm);
    }
}
