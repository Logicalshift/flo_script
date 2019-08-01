use super::core::*;
use super::super::symbol::*;
use super::super::editor::*;

use desync::{Desync, pipe_in};
use futures::*;
use futures::sync::oneshot;

use std::sync::*;

lazy_static! {
    static ref FINISHED_EDITS: Desync<()> = Desync::new(());
}

///
/// Actions available for editing/setting up a Gluon script host
///
#[derive(Clone, PartialEq, Debug)]
pub enum GluonScriptEdit {
    /// A standard script editing action
    ScriptEdit(ScriptEdit),

    /// Sets whether or not I/O expressions are evaluated
    SetRunIo(bool)
}

///
/// The editor for a Gluon script host
///
pub struct GluonScriptEditor {
    /// The core of the host that this editor represents
    core: Arc<Desync<GluonScriptHostCore>>
}

impl GluonScriptEditor {
    ///
    /// Creates a new script editor
    ///
    pub (crate) fn new(core: Arc<Desync<GluonScriptHostCore>>) -> GluonScriptEditor {
        GluonScriptEditor { core }
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

impl FloScriptEditor for GluonScriptEditor {
    ///
    /// Waits for edits from the specified stream and performs them as they arrive. Returns a future that indicates when the stream
    /// has been consumed.
    /// 
    /// Multiple edits can be sent at once to the script editor if needed: if this occurs, the streams are multiplexed and they are
    /// performed in any order.
    ///
    fn send_edits<Edits: 'static+Send+Stream<Item=ScriptEdit, Error=()>>(&self, edits: Edits) -> Box<dyn Future<Item=(), Error=()>> {
        // Turn into Gluon edits
        let edits           = edits.map(|edit| GluonScriptEdit::ScriptEdit(edit));

        // Send to the editor
        let finished_edits  = self.send_gluon_edits(edits);

        // Change the error to one we know
        let finished_edits  = finished_edits.map_err(|_canceled| ());

        Box::new(finished_edits)
    }
}
