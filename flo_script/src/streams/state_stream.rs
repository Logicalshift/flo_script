use super::input_stream_core::*;

use futures::*;
use futures::task;

use std::sync::*;

///
/// Represents a stream of state updates
///
/// A state stream differs from an input stream in that it will skip 'missed' inputs: that is, it assumes
/// that readers are only interested in reading the latest state and not in receiving out-of-date states.
/// This is useful for tasks such as updating a user interface, where the user needs to see the latest
/// state only and isn't interested in previous states.
///
pub struct StateStream<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> {
    /// The ID of this input stream
    stream_id:  usize,

    /// The core where this stream will read from
    core:       Arc<InputStreamCore<Symbol, Source>>
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> StateStream<Symbol, Source> {
    ///
    /// Creates a new input stream that will read from the specified core
    /// 
    pub fn new(core: Arc<InputStreamCore<Symbol, Source>>) -> StateStream<Symbol, Source> {
        let stream_id = core.allocate_state_stream();

        StateStream {
            stream_id:  stream_id,
            core:       core
        }
    }
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> Drop for StateStream<Symbol, Source> {
    fn drop(&mut self) {
        // Release this stream from the core
        let stream_id = self.stream_id;
        self.core.deallocate_stream(stream_id);
    }
}

impl<Symbol: 'static+Clone+Send, Source: 'static+Send+Stream<Item=Symbol, Error=()>> Stream for StateStream<Symbol, Source> {
    type Item   = Symbol;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Symbol>, ()> {
        // It's necessary to get the task here as the call to the core might end up on another thread
        let task = task::current();
        self.core.poll_state(self.stream_id, task)
    }
}
