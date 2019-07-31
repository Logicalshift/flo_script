use super::core::*;

use desync::Desync;

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
}
