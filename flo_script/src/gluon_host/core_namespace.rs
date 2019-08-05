use gluon::*;

///
/// Represents a script namespace
///
pub struct GluonScriptNamespace {
    /// The current thread for generating streaming scripts (or none if it hasn't been created yet)
    streaming: Option<RootedThread>,

    /// The current thread for generating state updating scripts
    state: Option<RootedThread>
}

impl GluonScriptNamespace {
    ///
    /// Creates a new script namespace. The scripting VM is initially not started
    ///
    pub fn new() -> GluonScriptNamespace {
        GluonScriptNamespace {
            streaming:  None,
            state:      None
        }
    }
}
