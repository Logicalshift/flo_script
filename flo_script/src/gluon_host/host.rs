use super::core::*;
use super::editor::*;

use futures::*;
use desync::{Desync, pipe_in};

use std::sync::*;

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
    pub fn send_gluon_edits<EditStream: 'static+Send+Stream<Item=GluonScriptEdit, Error=()>>(&self, edits: EditStream) {
        let core = Arc::clone(&self.core);

        // Pipe the edits straight to the core
        pipe_in(core, edits, |core, edit| {
            if let Ok(edit) = edit {
                core.edit(edit);
            }
        });
    }
}
