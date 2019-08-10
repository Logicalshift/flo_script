use gluon::{RootedThread};
use gluon::compiler_pipeline::{CompileValue};
use gluon_base::ast::{SpannedExpr};
use gluon_base::symbol::{Symbol};

use std::sync::*;
use std::marker::PhantomData;

///
/// The state of a computing script
///
#[derive(Clone, Copy, PartialEq, Debug)]
enum ComputingScriptState {
    /// Script has never run before
    NeverRun,

    /// Script is waiting for the next value
    WaitingForNextValue,

    /// Script is running and we're waiting for the value
    Running,

    /// Script has completed (has run and no longer depends on anything from the namespace)
    Finished
}

///
/// A stream that pulls results from a computing script
///
#[derive(Clone)]
pub struct ComputingScriptStream<Item> {
    /// The root thread that we'll spawn from when we need to run
    root: Arc<RootedThread>,

    /// The script that this will run
    script: Arc<CompileValue<SpannedExpr<Symbol>>>,

    item: PhantomData<Item>
}

impl<Item> ComputingScriptStream<Item> {
    ///
    /// Creates a new computing thread that reads from the specified symbol
    ///
    pub fn new(thread: Arc<RootedThread>, script: Arc<CompileValue<SpannedExpr<Symbol>>>) -> ComputingScriptStream<Item> {
        ComputingScriptStream {
            root:   thread,
            script: script,
            item:   PhantomData
        }
    }
}