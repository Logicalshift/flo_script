use futures::*;
use futures::task::Task;
use desync::Desync;

use std::usize;
use std::sync::*;
use std::collections::{HashMap, VecDeque};

/// The default max buffer size for an input stream core
const DEFAULT_MAX_BUFFER_SIZE: usize = 256;

///
/// The data for a single stream generating output from this input
///
struct StreamData<Symbol> {
    /// Symbols that are waiting to be read on this stream
    buffer: VecDeque<Symbol>,

    /// The futures task that this stream is waiting on
    ready: Option<Task>
}

///
/// The data for a state stream
///
/// State streams are different from input streams in that they only report the most recent value,
/// and will return that value immediately even if it hasn't updated
///
struct StateData<Symbol> {
    /// The current symbol for this state
    current_symbol: Option<Symbol>,

    /// Task to wake the stream reading from this state
    ready: Option<Task>
}

///
/// The collection of streams and states stored by an input stream core
///
struct StreamBuffers<Symbol> {
    /// The streams for this core
    streams: HashMap<usize, StreamData<Symbol>>,

    /// The states for this core
    states: HashMap<usize, StateData<Symbol>>
}

///
/// The core shared between all streams generated from an input symbol
///
pub struct InputStreamCore<Symbol: Send, Source> {
    /// The stream that is the source for this core (or none if no stream is attached yet)
    source_stream: Option<Source>,

    /// The most recently read symbol from the source stream
    last_symbol: Option<Symbol>,

    /// Set to true if the source stream has finished
    stream_finished: bool,

    /// The identifier to attach to the next stream that wants to read from this core
    next_stream_id: usize,

    /// The maximum number of symbols to buffer between the different readers before refusing to read more symbols from the source
    max_buffer_size: usize,

    /// The buffers for the streams that are attached to this core
    buffers: Arc<Desync<StreamBuffers<Symbol>>>,

    /// Desync where we send notifications of updates when they happen 
    // (this avoids issues with recursion when polls generate other polls, so that streams or states is not in use when we notify of an update)
    notify: Arc<Desync<()>>
}

impl<Symbol: 'static+Clone+Send, Source: Send+Stream<Item=Symbol, Error=()>> InputStreamCore<Symbol, Source> {
    ///
    /// Creates a new input stream core
    ///
    pub fn new() -> InputStreamCore<Symbol, Source> {
        let buffers = StreamBuffers {
            streams: HashMap::new(),
            states: HashMap::new()
        };

        InputStreamCore {
            source_stream:      None,
            last_symbol:        None,
            next_stream_id:     0,
            max_buffer_size:    DEFAULT_MAX_BUFFER_SIZE,
            stream_finished:    false,
            buffers:            Arc::new(Desync::new(buffers)),
            notify:             Arc::new(Desync::new(()))
        }
    }

    ///
    /// Changes the stream that's associated with this input stream
    ///
    pub fn replace_stream(&mut self, new_stream: Source) {
       // Update the source
       self.source_stream   = Some(new_stream);
       self.stream_finished = false;
        
       // Wake all of the streams so they poll the new stream
       let notify = Arc::clone(&self.notify);
       self.buffers.desync(move |buffers| Self::wake_streams(notify, usize::MAX, buffers));
    }

    ///
    /// Allocates a new stream that will read from the input stream
    ///
    /// Streams are polled using the `poll_stream()` function and will receive every symbol sent to the
    /// input stream. If they're connected after the other streams have started reading, they will return
    /// symbols starting at the first one that's still in the buffer.
    ///
    pub fn allocate_stream(&mut self) -> usize {
        // Assign an ID to this stream
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        // Finish allocating the stream in the background
        self.buffers.desync(move |buffers| {
            // For a new stream, we'll return the same symbols from the stream with the most full buffer
            let mut buffer: Option<&VecDeque<_>> = None;
            for existing_stream_data in buffers.streams.values() {
                if existing_stream_data.buffer.len() > buffer.map(|buffer| buffer.len()).unwrap_or(0) {
                    buffer = Some(&existing_stream_data.buffer);
                }
            }

            let new_buffer = buffer.map(|buffer| buffer.clone()).unwrap_or_else(|| VecDeque::new());

            // Create the stream data
            let stream_data = StreamData {
                buffer: new_buffer,
                ready:  None
            };

            // Store ready for use            
            buffers.streams.insert(stream_id, stream_data);
        });

        stream_id
    }

    ///
    /// Frees a stream from this core
    ///
    pub fn deallocate_stream(&mut self, stream_id: usize) {
        self.buffers.desync(move |buffers| { 
            buffers.streams.remove(&stream_id); 
            buffers.states.remove(&stream_id);
        });
    }

    ///
    /// Allocates a new state stream, returning the stream ID
    ///
    /// State streams are polled using the `poll_state()` function. These streams only return the
    /// most recent symbol available from the input stream (for states, only the most recent state is
    /// relevant).
    ///
    pub fn allocate_new_state_stream(&mut self) -> usize {
        // Assign a stream ID for this state
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        let last_symbol = self.last_symbol.clone();

        // Set up the data structure
        self.buffers.desync(move |buffers| {
            // Set up the state data for this stream
            let new_state = StateData {
                current_symbol: last_symbol,
                ready:          None
            };

            // Create the new state structure
            buffers.states.insert(stream_id, new_state);
        });

        stream_id
    }

    ///
    /// Drains as many entries as possible from the specified stream to the target streams
    /// 
    /// Returns (new_data_available, stream_finished)
    ///
    fn drain_stream(stream: &mut Source, buffer_to: &mut StreamBuffers<Symbol>, max_buffer_size: usize) -> (bool, bool, Option<Symbol>) {
        // Determine the maximum number of symbols to load for the streams
        let biggest_stream_count    = buffer_to.streams.values().map(|stream_data| stream_data.buffer.len()).max().unwrap_or(0);
        if biggest_stream_count >= max_buffer_size { return (false, false, None); }
        let mut remaining_symbols   = max_buffer_size - biggest_stream_count;
        let mut received_symbols    = vec![];
        let mut new_data_available  = false;
        let mut stream_finished     = false;
        let mut last_symbol         = None;

        loop {
            // Stop once any of the receiving streams has a fullybuffer
            if remaining_symbols <= 0 { break; }

            // Poll for the next symbol until the stream finishes or indicates it's not ready
            match stream.poll() {
                Err(())                             => { break; }
                Ok(Async::NotReady)                 => { break; }
                Ok(Async::Ready(None))              => { stream_finished = true; break; }

                Ok(Async::Ready(Some(next_symbol))) => { 
                    remaining_symbols -= 1; 
                    received_symbols.push(next_symbol);
                }
            }
        }

        if received_symbols.len() > 0 {
            // Tell the caller that new data is available. It will need to notify all of the streams that are waiting
            new_data_available = true;

            // Set the last symbol
            last_symbol = received_symbols.last().cloned();

            // Add the received symbols to the buffers
            buffer_to.streams.values_mut().skip(1)
                .for_each(|stream_buffer| stream_buffer.buffer.extend(received_symbols.iter().cloned()));
            buffer_to.streams.values_mut().nth(0)
                .map(|stream_buffer| stream_buffer.buffer.extend(received_symbols));
        }

        (new_data_available, stream_finished, last_symbol)
    }

    ///
    /// New data has arrived: wake all of the streams attached to this core
    ///
    fn wake_streams(notify: Arc<Desync<()>>, stream_id: usize, buffers: &mut StreamBuffers<Symbol>) {
        // Work out the tasks that need notifications
        let to_notify = buffers.streams.iter_mut().flat_map(|(id, stream)| {
            if *id != stream_id {
                stream.ready.take()
            } else {
                None
            }
        }).collect::<Vec<_>>();

        // Send to the notification desync
        if to_notify.len() > 0 {
            notify.desync(move |_| {
                to_notify.into_iter().for_each(|task| task.notify())
            });
        }
    }

    ///
    /// Updates the last symbol associated with this stream
    ///
    fn update_last_symbol(&mut self, last_symbol: Symbol, stream_id: usize, buffers: &mut StreamBuffers<Symbol>) {
        // Update all of the active symbols for all of the states
         buffers.states.values_mut()
            .for_each(|state| {
                state.current_symbol = Some(last_symbol.clone());
            });
        self.last_symbol    = Some(last_symbol);

        // Collect the states to notify
        let to_notify       = buffers.states.iter_mut()
            .flat_map(|(id, state)| {
                if *id != stream_id {
                    state.ready.take()
                } else {
                    None
                }
            }).collect::<Vec<_>>();

        // Perform the notifications in the background
        if to_notify.len() > 0 {
            self.notify.desync(move |_| {
                to_notify.into_iter().for_each(|task| task.notify());
            });
        }
    }

    ///
    /// Polls the stream with a particular ID (from a future or a stream)
    ///
    pub fn poll_stream(&mut self, stream_id: usize, poll_task: Task) -> Poll<Option<Symbol>, ()> {
        // Clone the stream reference to get around some Rust book-keeping (it assumes all of 'self' is borrowed in the closure if we don't do this)
        let buffers = Arc::clone(&self.buffers);

        // As sync can potentially run on a separate thread, get the active task before acting on the streams
        let task    = poll_task;

        buffers.sync(|buffers| {
            // If the stream has buffered data waiting, just return that
            if let Some(stream) = buffers.streams.get_mut(&stream_id) {
                // Any task for this stream is now invalid
                stream.ready.take();

                if let Some(next_symbol) = stream.buffer.pop_front() {
                    // Just return straight from the buffer while there is some
                    return Ok(Async::Ready(Some(next_symbol)));
                }
            }

            // TODO: only the stream that is being polled will be notified when the source stream has more data available right now
            // This may block other threads if it does not respond (ideally we should always notify all of the streams when the source stream notifies)

            // Read as many symbols as we can from the source stream and buffer them (this avoids too much round-robin signalling)
            let max_buffer_size                                 = self.max_buffer_size; 
            let (new_data_available, finished, new_last_symbol) = self.source_stream.as_mut()
                .map(|source_stream| Self::drain_stream(source_stream, buffers, max_buffer_size))
                .unwrap_or((false, false, None));

            // Update the last symbol if there's a new one
            new_last_symbol.map(|new_last_symbol| self.update_last_symbol(new_last_symbol, stream_id, buffers));

            // Mark as finished if the source stream is done
            if finished {
                self.stream_finished = true;
            }

            // Wake all of the other streams if new data has been loaded from the source stream
            if new_data_available {
                Self::wake_streams(Arc::clone(&self.notify), stream_id, buffers);
            }

            // Buffer the next symbol
            if let Some(mut stream) = buffers.streams.get_mut(&stream_id) {
                // Try to read the next symbol from the current stream
                if let Some(next_symbol) = stream.buffer.pop_front() {
                    return Ok(Async::Ready(Some(next_symbol)));
                } else if self.stream_finished {
                    // If the source stream is done and the buffer is empty, then this stream has finished too
                    return Ok(Async::Ready(None));
                } else {
                    // If there is nothing in the buffer then we need to wait for the source stream
                    stream.ready = Some(task);
                    return Ok(Async::NotReady);
                }
            }

            // Streams whose ID doesn't exist return no data
            Ok(Async::Ready(None))
        })
    }

    ///
    /// Polls a state stream for the next update
    ///
    pub fn poll_state(&mut self, stream_id: usize, poll_task: Task) -> Poll<Option<Symbol>, ()> {
        let buffers = Arc::clone(&self.buffers);

        buffers.sync(|buffers| {
            if let Some(state) = buffers.states.get_mut(&stream_id) {
                state.ready.take();

                if let Some(value) = state.current_symbol.take() {
                    // Return the current value if there is one
                    return Ok(Async::Ready(Some(value)));
                }
            }

            // Return immediately if the input stream is finished
            if self.stream_finished {
                return Ok(Async::Ready(None));
            }

            // Read as many symbols as we can from the source stream and buffer them (this avoids too much round-robin signalling)
            let max_buffer_size                                 = self.max_buffer_size; 
            let (new_data_available, finished, new_last_symbol) = self.source_stream.as_mut()
                .map(|source_stream| Self::drain_stream(source_stream, buffers, max_buffer_size))
                .unwrap_or((false, false, None));

            // Update the last symbol if there's a new one
            new_last_symbol.map(|new_last_symbol| self.update_last_symbol(new_last_symbol, stream_id, buffers));

            // Mark as finished if the source stream is done
            if finished {
                self.stream_finished = true;
            }

            // Wake all of the other streams if new data has been loaded from the source stream
            if new_data_available {
                Self::wake_streams(Arc::clone(&self.notify), stream_id, buffers);
            }

            // Try to read the value from the stream again
            if let Some(state) = buffers.states.get_mut(&stream_id) {
                state.ready.take();

                if let Some(value) = state.current_symbol.take() {
                    // Return the current value if there is one
                    return Ok(Async::Ready(Some(value)));
                } else if !self.stream_finished {
                    // Wait for the next state update
                    state.ready = Some(poll_task);
                    return Ok(Async::NotReady);
                } else {
                    // Stream is finished: there will be no more states
                    return Ok(Async::Ready(None));
                }
            }

            // Not a state stream, so we just stop immediately
            Ok(Async::Ready(None))
        })
    }
}
