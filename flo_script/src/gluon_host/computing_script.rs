use super::derived_state::*;
use super::super::error::*;

use gluon::{RootedThread, Compiler};
use gluon::compiler_pipeline::{CompileValue, Executable};
use gluon::vm::api::{VmType, Getable};
use gluon::base::ast::{SpannedExpr};
use gluon::base::symbol::{Symbol};
use futures::*;

use std::mem;
use std::sync::*;
use std::marker::PhantomData;

///
/// The state of a computing script
///
enum ComputingScriptState<Item> {
    /// Script is running and will produce a simple result
    GeneratingResult(Box<dyn Future<Item=Item, Error=gluon::Error>+Send>),

    /// Script has completed (has run and no longer depends on anything from the namespace)
    Finished
}

///
/// A stream that pulls results from a computing script
///
pub struct ComputingScriptStream<Item> {
    /// The current state of the computing script
    state: ComputingScriptState<Item>,

    /// The root thread that we'll spawn from when we need to run
    root: Arc<RootedThread>,

    /// The compiler that created the script
    compiler: Arc<Mutex<Compiler>>,

    /// We don't actually store any item of the specified data type
    item: PhantomData<Item>
}

impl<'vm, Item> ComputingScriptStream<Item> 
where   DerivedState<Item>: VmType,
        Item:               for<'value> Getable<'vm, 'value> + VmType + Send + 'static {
    ///
    /// Creates a new computing thread that reads from the specified symbol
    ///
    pub fn new(root_thread: Arc<RootedThread>, script: CompileValue<SpannedExpr<Symbol>>, compiler: Compiler) -> FloScriptResult<ComputingScriptStream<Item>> {
        let symbol_type         = Item::make_type(&*root_thread);
        let derived_state_type  = DerivedState::<Item>::make_type(&*root_thread);
        let mut compiler        = compiler;

        let initial_state = if script.typ == symbol_type {
            // Computed expression with no dependencies
            let root_copy       = Arc::clone(&root_thread);
            let thread          = root_thread.new_thread().expect("script thread");
            let future_result   = script.run_expr(&mut compiler, thread, "", "", ())
                .map(move |result| Item::from_value(&*root_copy, result.value.get_variant()));

            ComputingScriptState::GeneratingResult(Box::new(future_result))
        } else if script.typ == derived_state_type {
            // Computed expression with dependencies
            ComputingScriptState::Finished
        } else {
            // Not a valid type
            return Err(FloScriptError::IncorrectType);
        };

        Ok(ComputingScriptStream {
            root:           root_thread,
            compiler:       Arc::new(Mutex::new(compiler)),
            state:          initial_state,
            item:           PhantomData
        })
    }
}

impl<'vm, Item> ComputingScriptStream<Item> 
where   Item: for<'value> Getable<'vm, 'value> + VmType + Send + 'static {
    ///
    /// Given a script in the 'GeneratingResult' state, 
    ///
    fn poll_for_simple_result(mut future_result: Box<dyn Future<Item=Item, Error=gluon::Error>+Send>) -> (ComputingScriptState<Item>, Poll<Option<Item>, ()>) {
        use self::ComputingScriptState::*;

        match future_result.poll() {
            Ok(Async::NotReady)         => (GeneratingResult(future_result), Ok(Async::NotReady)),
            Ok(Async::Ready(result))    => (Finished, Ok(Async::Ready(Some(result)))),
            Err(_err)                   => (Finished, Err(()))
        }
    }
}

impl<'vm, Item> Stream for ComputingScriptStream<Item>
where   Item: for<'value> Getable<'vm, 'value> + VmType + Send + 'static {
    type Item = Item;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Item>, ()> {
        use self::ComputingScriptState::*;

        // Steal the current state of the stream (we'll wind up in the finished state if there's a panic or something)
        let mut current_state = Finished;
        mem::swap(&mut current_state, &mut self.state);

        // Dispatch the next action based on the current script state
        let (new_state, result) = match current_state {
            GeneratingResult(future_result) => Self::poll_for_simple_result(future_result),
            Finished                        => (Finished, Ok(Async::Ready(None)))
        };

        // Update to the new state
        self.state = new_state;

        result
    }
}
