use gluon::*;

///
/// Core of a script host that targets the Gluon scripting language
/// 
/// See [https://gluon-lang.org] for details on this language.
///
pub struct GluonScriptHostCore {
    /// The primary VM thread
    root_thread: RootedThread
}

impl GluonScriptHostCore {
    ///
    /// Creates a new script core
    ///
    pub fn new() -> GluonScriptHostCore {
        let root_thread = new_vm();

        GluonScriptHostCore { root_thread }
    }
}
