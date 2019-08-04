use gluon::{RootedThread, Thread};
use gluon_vm::{ExternModule, Result};
use gluon_vm::api::{FunctionRef, OpaqueValue};
use gluon_vm::api::generic::{A, B};

type ValueA = OpaqueValue<RootedThread, A>;
type ValueB = OpaqueValue<RootedThread, B>;

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
    value: TValue
}

impl<TValue> State<TValue> {
    ///
    /// Creates a new state that is not dependent on any input states
    ///
    pub fn new(value: TValue) -> State<TValue> {
        State { 
            value: value
        }
    }
}

///
/// Implementation of flat_map for the State monad
///
fn flat_map(mut a: FunctionRef<fn(ValueA) -> State<ValueB>>, b: State<ValueA>) -> State<ValueB> {
    let value_a = b.value;
    let value_b = a.call(value_a).unwrap();

    value_b
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
