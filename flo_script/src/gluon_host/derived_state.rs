use super::super::symbol::*;

use gluon::{RootedThread, Thread};
use gluon::vm::{ExternModule, Result};
use gluon::vm::api::{FunctionRef, OpaqueValue, UserdataValue};
use gluon::vm::api::generic::{A, B};

use std::iter;
use std::sync::*;
use std::collections::HashSet;

type ValueA = OpaqueValue<RootedThread, A>;
type ValueB = OpaqueValue<RootedThread, B>;

///
/// Structs come with some user data
///
#[derive(Userdata, VmType, Trace, Clone, Debug)]
#[gluon_trace(skip)]
#[gluon(vm_type = "flo.computed.DerivedStateDependencies")]
struct DerivedStateDependencies {
    dependencies: Arc<HashSet<FloScriptSymbol>>
}

impl DerivedStateDependencies {
    ///
    /// Creates a new (empty) state dependencies structure
    ///
    fn new() -> DerivedStateDependencies {
        DerivedStateDependencies {
            dependencies: Arc::new(HashSet::new())
        }
    }

    ///
    /// Creates a new state dependencies structure containing a single dependency ID
    ///
    fn with_dependency(dependency: FloScriptSymbol) -> DerivedStateDependencies {
        DerivedStateDependencies {
            dependencies: Arc::new(iter::once(dependency).collect())
        }
    }

    ///
    /// Merge these dependencies with the dependencies from another state
    ///
    fn merge_with(&mut self, merge_with: &DerivedStateDependencies) {
        if merge_with.dependencies.len() == 0 {
            // Nothing to do
        } else if self.dependencies.len() == 0 {
            // Just take the dependencies from the other side
            self.dependencies = Arc::clone(&merge_with.dependencies);
        } else {
            // Create a new dependencies collection
            self.dependencies = if self.dependencies.len() < merge_with.dependencies.len() {
                // Clone the larger dependency set, then insert from the smaller
                let mut new_dependencies = (*merge_with.dependencies).clone();
                for dep in self.dependencies.iter() {
                    new_dependencies.insert(*dep);
                }

                Arc::new(new_dependencies)
            } else {
                // Clone the larger dependency set, then insert from the smaller
                let mut new_dependencies = (*self.dependencies).clone();
                for dep in merge_with.dependencies.iter() {
                    new_dependencies.insert(*dep);
                }

                Arc::new(new_dependencies)
            }
        }
    }
}

///
/// Monad representing a state value
/// 
/// A state value carries along with the symbols that were read from. We use this later on to decide
/// what to re-evaluate when new data arrives via an input stream.
///
#[derive(VmType, Getable, Pushable)]
pub struct DerivedState<TValue> {
    /// The value of this state
    value:          TValue,

    // User data for this state
    dependencies:   UserdataValue<DerivedStateDependencies>
}

impl<TValue> DerivedState<TValue> {
    ///
    /// Creates a new state that is not dependent on any input states
    ///
    pub fn new(value: TValue) -> DerivedState<TValue> {
        DerivedState { 
            value:          value,
            dependencies:   UserdataValue(DerivedStateDependencies::new())
        }
    }

    ///
    /// Creates a new state with a single dependency
    ///
    pub fn with_dependency(value: TValue, dependency: FloScriptSymbol) -> DerivedState<TValue> {
        DerivedState { 
            value:          value,
            dependencies:   UserdataValue(DerivedStateDependencies::with_dependency(dependency))
        }        
    }

    ///
    /// Merges the dependencies from another state to update this state
    ///
    fn merge_dependencies(self, merge_with: UserdataValue<DerivedStateDependencies>) -> DerivedState<TValue> {
        let mut new_state = self;

        let UserdataValue(ref mut our_dependencies) = new_state.dependencies;
        let UserdataValue(merge_with)               = merge_with;

        our_dependencies.merge_with(&merge_with);

        new_state
    }
}

///
/// Implementation of flat_map for the State monad
///
fn flat_map(mut a: FunctionRef<fn(ValueA) -> DerivedState<ValueB>>, b: DerivedState<ValueA>) -> DerivedState<ValueB> {
    let value_a = b.value;
    let deps_a  = b.dependencies;
    let value_b = a.call(value_a).unwrap();

    value_b.merge_dependencies(deps_a)
}

///
/// Wraps a value in a new state
///
fn wrap(a: ValueA) -> DerivedState<ValueA> {
    DerivedState::new(a)
}

///
/// Generates the flo.state extern module for a Gluon VM
///
pub fn load(vm: &Thread) -> Result<ExternModule> {
    vm.register_type::<DerivedStateDependencies>("flo.computed.DerivedStateDependencies", &[])?;
    vm.register_type::<DerivedState<A>>("flo.computed.DerivedState", &["a"])?;

    ExternModule::new(vm, record! {
        type DerivedState a                 => DerivedState<A>,
        type DerivedStateDependencies       => DerivedStateDependencies,

        wrap                                => primitive!(1, wrap),
        flat_map                            => primitive!(2, flat_map)
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
        import::add_extern_module(&vm, "flo.computed", load);

        let _some_type = primitives::DirEntry::make_type(&vm);
        let _some_type = DerivedState::<i32>::make_type(&vm);
    }
}
