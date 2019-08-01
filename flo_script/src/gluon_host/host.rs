use super::core::*;
use super::editor::*;

use futures::*;
use futures::stream;
use futures::sync::oneshot;
use desync::{Desync, pipe_in};

use std::sync::*;

lazy_static! {
    static ref FINISHED_EDITS: Desync<()> = Desync::new(());
}

///
/// A script host for Gluon scripts
/// 
/// See [https://gluon-lang.org] for details on this language.
///
pub struct GluonScriptHost {
    /// The core is used to execute the scripts asynchronously and process their results 
    core: Arc<Desync<GluonScriptHostCore>>
}

impl GluonScriptHost {
    /// 
    /// Creates a new Gluon script host with no scripts running
    /// 
    pub fn new() -> GluonScriptHost {
        let core = GluonScriptHostCore::new();

        GluonScriptHost {
            core: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Receives edits from the specified streams
    /// 
    /// The returned future indicates when the stream is consumed. It can signal that it's cancelled in the event that the stream is
    /// not completely consumed before the script host is dropped.
    ///
    pub fn send_gluon_edits<EditStream: 'static+Send+Stream<Item=GluonScriptEdit, Error=()>>(&self, edits: EditStream) -> impl Future<Item=(), Error=oneshot::Canceled> {
        let core = Arc::clone(&self.core);

        // Create a future to notify when the stream is finished
        let (notify_finished, finished) = oneshot::channel();

        // Alter the edit stream so it notifies the future once it's exhausted
        let mut notify_finished         = Some(notify_finished);
        let mut edits                   = edits;
        let edits                       = stream::poll_fn(move || {
            let edit = edits.poll();

            match edit {
                Ok(Async::Ready(None)) => {
                    notify_finished.take().map(|notify_finished| FINISHED_EDITS.desync(move |_| { notify_finished.send(()).ok(); }));
                    Ok(Async::Ready(None))
                },

                _ => edit
            }
        });

        // Pipe the edits straight to the core
        pipe_in(core, edits, |core, edit| {
            if let Ok(edit) = edit {
                core.edit(edit);
            }
        });

        // Result is a future indicating when we've exhausted the stream (it'll signal cancelled if we stop polling the stream, which will happen if the host is dropped before it completes)
        finished
    }
}
