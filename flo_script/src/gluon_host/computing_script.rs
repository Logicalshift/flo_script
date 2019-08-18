use super::derived_state::*;
use super::super::error::*;

use gluon::{RootedThread, Compiler};
use gluon::compiler_pipeline::{CompileValue, Executable};
use gluon::vm::api::{VmType, Getable};
use gluon::base::ast::{SpannedExpr};
use gluon::base::symbol::{Symbol};
use futures::*;

use std::sync::*;
use std::marker::PhantomData;

///
/// The state of a computing script
///
#[derive(Clone, Copy, PartialEq, Debug)]
enum ComputingScriptState {
    /// Script has never run before
    NeverRun,

    /// Script is running and we're waiting for the value
    Running,

    /// Script has finished and is waiting for the next value
    WaitingForNextValue,

    /// Script has completed (has run and no longer depends on anything from the namespace)
    Finished
}

///
/// How to interpret the result of the computed script
///
#[derive(Clone, Copy, PartialEq, Debug)]
enum ComputingScriptResultType {
    /// A value that's directly computed
    StraightValue,

    /// A value that's derived from other state
    DerivedValue
}

///
/// A stream that pulls results from a computing script
///
#[derive(Clone)]
pub struct ComputingScriptStream<Item> {
    /// The current state of the computing script
    state: ComputingScriptState,

    /// The root thread that we'll spawn from when we need to run
    root: Arc<RootedThread>,

    /// The script that this will run
    script: Arc<CompileValue<SpannedExpr<Symbol>>>,

    /// The compiler that created the script
    compiler: Option<Arc<Mutex<Compiler>>>,

    /// The type to look for in the result
    result_type: ComputingScriptResultType,

    item: PhantomData<Item>
}

impl<Item: 'static+VmType> ComputingScriptStream<Item> 
where DerivedState<Item>: VmType {
    ///
    /// Creates a new computing thread that reads from the specified symbol
    ///
    pub fn new(thread: Arc<RootedThread>, script: Arc<CompileValue<SpannedExpr<Symbol>>>, compiler: Compiler) -> FloScriptResult<ComputingScriptStream<Item>> {
        let symbol_type         = Item::make_type(&*thread);
        let derived_state_type  = DerivedState::<Item>::make_type(&*thread);

        let result_type = if script.typ == symbol_type {
            // Computed expression with no dependencies
            ComputingScriptResultType::StraightValue
        } else if script.typ == derived_state_type {
            // Computed expression with dependencies
            ComputingScriptResultType::DerivedValue
        } else {
            // Not a valid type
            return Err(FloScriptError::IncorrectType);
        };

        Ok(ComputingScriptStream {
            root:           thread,
            script:         script,
            result_type:    result_type,
            compiler:       Some(Arc::new(Mutex::new(compiler))),
            state:          ComputingScriptState::NeverRun,
            item:           PhantomData
        })
    }
}

impl<'vm, Item> ComputingScriptStream<Item> 
where   Item: for<'value> Getable<'vm, 'value> + VmType + Send + 'vm {
    fn poll_start_script(&mut self) -> Poll<Option<Item>, ()> {
        unimplemented!()
    }

    fn poll_script_running(&mut self) -> Poll<Option<Item>, ()> {
        unimplemented!()
    }

    fn poll_wait_for_value(&mut self) -> Poll<Option<Item>, ()> {
        unimplemented!()
    }
}

impl<'vm, Item> Stream for ComputingScriptStream<Item>
where   Item: for<'value> Getable<'vm, 'value> + VmType + Send + 'vm {
    type Item = Item;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Item>, ()> {
        use self::ComputingScriptState::*;

        match self.state {
            NeverRun            => self.poll_start_script(),
            Running             => self.poll_script_running(),
            WaitingForNextValue => self.poll_wait_for_value(),
            Finished            => Ok(Async::Ready(None))
        }
    }
}
