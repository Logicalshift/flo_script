use futures::*;
use futures::task::Task;
use desync::Desync;

use std::sync::*;
use std::collections::{HashMap, VecDeque};

/// The default max buffer size for an input stream core
const DEFAULT_MAX_BUFFER_SIZE: usize = 256;

///
/// The data for a single stream being used as input
///
struct StreamData<Symbol> {
    /// Symbols that are waiting to be read on this stream
    buffer: VecDeque<Symbol>,

    /// The futures task that this stream is waiting on
    ready: Option<Task>
}

///
/// The core shared between all streams generated from an input symbol
///
pub struct InputStreamCore<Symbol: Send, Source> {
    /// The stream that is the source for this core
    source_stream: Source,

    /// Set to true if the source stream has finished
    stream_finished: bool,

    /// The identifier to attach to the next stream that wants to read from this core
    next_stream_id: usize,

    /// The maximum number of symbols to buffer between the different readers before refusing to read more symbols from the source
    max_buffer_size: usize,

    /// The streams that are attached to this core
    streams: Arc<Desync<HashMap<usize, StreamData<Symbol>>>>
}

impl<Symbol: 'static+Clone+Send, Source: Send+Stream<Item=Symbol, Error=()>> InputStreamCore<Symbol, Source> {
    ///
    /// Creates a new input stream core
    ///
    pub fn new(source: Source) -> InputStreamCore<Symbol, Source> {
        InputStreamCore {
            source_stream:      source,
            next_stream_id:     0,
            max_buffer_size:    DEFAULT_MAX_BUFFER_SIZE,
            stream_finished:    false,
            streams:            Arc::new(Desync::new(HashMap::new()))
        }
    }

    ///
    /// Changes the stream that's associated with this input stream
    ///
    pub fn replace_stream(&mut self, new_stream: Source) {
       // Update the source
       self.source_stream   = new_stream;
       self.stream_finished = false;
        
       // Wake all of the streams so they poll the new stream
       self.streams.desync(|streams| { streams.values_mut().for_each(|stream| { stream.ready.take().map(|ready| ready.notify()); }) });
    }

    ///
    /// Allocates a new stream that will read from the input stream (starting at the most recent symbol)
    ///
    pub fn allocate_stream(&mut self) -> usize {
        // Assign an ID to this stream
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        // Create the stream data
        let stream_data = StreamData {
            buffer: VecDeque::new(),
            ready:  None
        };

        self.streams.desync(move |streams| { streams.insert(stream_id, stream_data); });

        stream_id
    }

    ///
    /// Frees a stream from this core
    ///
    pub fn deallocate_stream(&mut self, stream_id: usize) {
        self.streams.desync(move |streams| { streams.remove(&stream_id); });
    }

    ///
    /// Polls the stream with a particular ID (from a future or a stream)
    ///
    pub fn poll_stream(&mut self, stream_id: usize) -> Poll<Option<Symbol>, ()> {
        let streams = Arc::clone(&self.streams);

        streams.sync(|streams| {
            // If the stream has buffered data waiting, just return that
            if let Some(stream) = streams.get_mut(&stream_id) {
                // Any task for this stream is now invalid
                stream.ready.take();

                if stream.buffer.len() > 0 {
                    // Just return straight from the buffer while there is some
                    return Ok(Async::Ready(Some(stream.buffer.pop_front().unwrap())));
                }
            }

            // Stall if any streams have a full buffer
            if streams.values().any(|stream| stream.buffer.len() >= self.max_buffer_size) {
                streams.get_mut(&stream_id).map(|stream| stream.ready = Some(task::current()));
                return Ok(Async::NotReady);
            }

            // Buffer the next symbol
            if let Some(mut stream) = streams.get_mut(&stream_id) {
                // Reached the end of the stream if stream_finished is true
                if self.stream_finished {
                    return Ok(Async::Ready(None));
                }

                // Fetch the next symbol from the stream
                let next_symbol = self.source_stream.poll();

                return match next_symbol {
                    Ok(Async::Ready(Some(next_symbol))) => {
                        // Buffer the next symbol for all of the other streams
                        streams.iter_mut().for_each(|(id, stream)| {
                            if *id != stream_id {
                                stream.buffer.push_back(next_symbol.clone());
                            }
                        });

                        Ok(Async::Ready(Some(next_symbol)))
                    },

                    Ok(Async::Ready(None)) => {
                        // Stream has finished
                        self.stream_finished = true;
                        Ok(Async::Ready(None))
                    }

                    Err(()) => {
                        // Stream isn't really supposed to produce any errors. We just relay these directly
                        Err(())
                    },

                    Ok(Async::NotReady) => {
                        // Stream is not ready. Remember the task
                        stream.ready = Some(task::current());

                        // TODO: When the current task notifies also wake up the other streams

                        Ok(Async::NotReady)
                    },
                };
            }

            // Streams whose ID doesn't exist return no data
            Ok(Async::Ready(None))
        })
    }
}
