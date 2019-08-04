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

///
/// Implementation of flat_map for the State monad
///
fn flat_map(mut a: FunctionRef<fn(ValueA) -> State<ValueB>>, b: State<ValueA>) -> State<ValueB> {
    let value_a = b.value;
    let value_b = a.call(value_a).unwrap();

    value_b
}

///
/// Generates the flo.state extern module for a Gluon VM
///
pub fn load(vm: &Thread) -> Result<ExternModule> {
    ExternModule::new(vm, record! {
        type flo::state::State a => State<A>,

        flat_map => primitive!(2, flat_map)
    })
}
