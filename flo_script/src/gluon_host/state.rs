use gluon::{RootedThread, Thread};
use gluon_vm::{ExternModule, Result};
use gluon_vm::api::{FunctionRef, OpaqueValue, UserdataValue};
use gluon_vm::api::generic::{A, B};

use std::iter;
use std::sync::*;
use std::collections::HashSet;

type ValueA = OpaqueValue<RootedThread, A>;
type ValueB = OpaqueValue<RootedThread, B>;

///
/// Structs come with some user data
///
#[derive(Clone, Debug, Trace, Userdata)]
#[gluon_trace(skip)]
struct StateDependencies {
    dependencies: Arc<HashSet<u64>>
}

impl StateDependencies {
    ///
    /// Creates a new (empty) state dependencies structure
    ///
    fn new() -> StateDependencies {
        StateDependencies {
            dependencies: Arc::new(HashSet::new())
        }
    }

    ///
    /// Creates a new state dependencies structure containing a single dependency ID
    ///
    fn with_dependency(dependency: u64) -> StateDependencies {
        StateDependencies {
            dependencies: Arc::new(iter::once(dependency).collect())
        }
    }

    ///
    /// Merge these dependencies with the dependencies from another state
    ///
    fn merge_with(&mut self, merge_with: &StateDependencies) {
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
/// When state values are requested from the environment, we remember what was requested so we know to
/// re-evaluate the expression when the state changes
///
#[derive(VmType, Getable, Pushable)]
#[gluon(vm_type = "flo.state.State")]
pub struct State<TValue> {
    /// The value of this state
    value: TValue,

    // User data for this state
    dependencies: UserdataValue<StateDependencies>
}

impl<TValue> State<TValue> {
    ///
    /// Creates a new state that is not dependent on any input states
    ///
    pub fn new(value: TValue) -> State<TValue> {
        State { 
            value:          value,
            dependencies:   UserdataValue(StateDependencies::new())
        }
    }

    ///
    /// Creates a new state with a single dependency
    ///
    pub fn with_dependency(value: TValue, dependency: u64) -> State<TValue> {
        State { 
            value:          value,
            dependencies:   UserdataValue(StateDependencies::with_dependency(dependency))
        }        
    }

    ///
    /// Merges the dependencies from another state to update this state
    ///
    fn merge_dependencies(self, merge_with: UserdataValue<StateDependencies>) -> State<TValue> {
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
fn flat_map(mut a: FunctionRef<fn(ValueA) -> State<ValueB>>, b: State<ValueA>) -> State<ValueB> {
    let value_a = b.value;
    let deps_a  = b.dependencies;
    let value_b = a.call(value_a).unwrap();

    value_b.merge_dependencies(deps_a)
}

///
/// Wraps a value in a new state
///
fn wrap(a: ValueA) -> State<ValueA> {
    State::new(a)
}

///
/// Generates the flo.state extern module for a Gluon VM
///
pub fn load(vm: &Thread) -> Result<ExternModule> {
    ExternModule::new(vm, record! {
        type flo::state::State a    => State<A>,

        wrap                        => primitive!(1, wrap),
        flat_map                    => primitive!(2, flat_map)
    })
}
