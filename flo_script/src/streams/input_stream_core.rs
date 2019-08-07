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
    /// Drains as many entries as possible from the specified stream to the target streams
    /// 
    /// Returns (new_data_available, stream_finished)
    ///
    fn drain_stream(stream: &mut Source, buffer_to: &mut HashMap<usize, StreamData<Symbol>>, max_buffer_size: usize) -> (bool, bool) {
        // Determine the maximum number of symbols to load for the streams
        let biggest_stream_count    = buffer_to.values().map(|stream_data| stream_data.buffer.len()).max().unwrap_or(0);
        if biggest_stream_count >= max_buffer_size { return (false, false); }
        let mut remaining_symbols   = max_buffer_size - biggest_stream_count;
        let mut received_symbols    = vec![];
        let mut new_data_available  = false;
        let mut stream_finished     = false;

        loop {
            // Stop once any of the receiving streams has a fullybuffer
            if remaining_symbols <= 0 { break; }

            // Poll for the next symbol until the stream finishes or indicates it's not ready
            match stream.poll() {
                Err(())                             => { break; }
                Ok(Async::NotReady)                 => { break; }
                Ok(Async::Ready(Some(next_symbol))) => { remaining_symbols -= 1; received_symbols.push(next_symbol); }
                Ok(Async::Ready(None))              => { stream_finished = true; break; }
            }
        }

        if received_symbols.len() > 0 {
            // Tell the caller that new data is available. It will need to notify all of the streams that are waiting
            new_data_available = true;

            // Add the received symbols to the buffers
            buffer_to.values_mut().skip(1)
                .for_each(|stream_buffer| stream_buffer.buffer.extend(received_symbols.iter().cloned()));
            buffer_to.values_mut().nth(0)
                .map(|stream_buffer| stream_buffer.buffer.extend(received_symbols));
        }

        (new_data_available, stream_finished)
    }

    ///
    /// Polls the stream with a particular ID (from a future or a stream)
    ///
    pub fn poll_stream(&mut self, stream_id: usize) -> Poll<Option<Symbol>, ()> {
        // Clone the stream reference to get around some Rust book-keeping (it assumes all of 'self' is borrowed in the closure if we don't do this)
        let streams = Arc::clone(&self.streams);

        // As sync can potentially run on a separate thread, get the active task before acting on the streams
        let task    = task::current();

        streams.sync(|mut streams| {
            // If the stream has buffered data waiting, just return that
            if let Some(stream) = streams.get_mut(&stream_id) {
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
            let (new_data_available, finished) = Self::drain_stream(&mut self.source_stream, &mut streams, self.max_buffer_size);

            // Mark as finished if the source stream is done
            if finished {
                self.stream_finished = true;
            }

            // Wake all of the other streams if new data has been loaded from the source stream
            if new_data_available {
                self.streams.desync(move |streams| { 
                    streams.iter_mut()
                        .for_each(|(id, stream)| { if *id != stream_id { stream.ready.take().map(|ready| ready.notify()); } }); 
                });
            }

            // Buffer the next symbol
            if let Some(mut stream) = streams.get_mut(&stream_id) {
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
}
