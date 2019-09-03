use super::core_namespace::*;
use super::super::symbol::*;

use futures::*;
use gluon::{Thread, Compiler};
use gluon::vm::{ExternModule, Result, Variants};
use gluon::vm::api::{VmType, FunctionRef, ValueRef, ActiveThread, Getable, Pushable, UserdataValue};
use gluon::vm::api::generic::{A};
use gluon::import;
use desync::{Desync};

use std::sync::*;
use std::result;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::collections::{HashSet, HashMap};
use std::any::{Any};

///
/// Data passed through the derived state monad
///
#[derive(Userdata, VmType, Trace)]
#[gluon_trace(skip)]
#[gluon(vm_type = "flo.computed.prim.DerivedStateData")]
pub struct DerivedStateData {
    /// The namespace that this state is for
    namespace: Arc<Desync<GluonScriptNamespace>>,

    /// The symbols that the last value of this state depended upon
    dependencies: HashSet<FloScriptSymbol>,

    /// The streams that are active in this state, mapped by the symbol they're active for
    active_streams: HashMap<FloScriptSymbol, Mutex<Box<dyn Any+Send>>>
}

/// Container type used so we can use 'Any' to get the stream of the appropriate type
struct StreamRef<TItem>(Box<dyn Stream<Item=TItem, Error=()>+Send>);

impl Clone for DerivedStateData {
    fn clone(&self) -> DerivedStateData {
        // TODO: active_streams?
        unimplemented!()
    }
}

impl DerivedStateData {
    ///
    /// Creates an entirely new blank derived state data object
    ///
    pub fn new(namespace: Arc<Desync<GluonScriptNamespace>>) -> DerivedStateData {
        DerivedStateData {
            namespace:      namespace,
            dependencies:   HashSet::new(),
            active_streams: HashMap::new()
        }
    }

    ///
    /// Retrieves the namespace for this state
    ///
    pub fn get_namespace(&self) -> Arc<Desync<GluonScriptNamespace>> {
        Arc::clone(&self.namespace)
    }

    ///
    /// Returns true if this state has an active stream for the specified symbol
    ///
    pub fn has_stream(&self, symbol: FloScriptSymbol) -> bool {
        self.active_streams.contains_key(&symbol)
    }

    ///
    /// Polls the stream for the specified symbol (returning None if the stream is not running)
    ///
    pub fn poll_stream<TStreamItem: 'static>(&mut self, symbol: FloScriptSymbol) -> Option<Poll<Option<TStreamItem>, ()>> {
        // Attempt to fetch the stream from the list of active streams
        if let Some(stream) = self.active_streams.get(&symbol) {
            // Make sure it's a stream of the 
            let mut stream = stream.lock().unwrap();
            if let Some(StreamRef(stream)) = stream.downcast_mut::<StreamRef<TStreamItem>>() {
                // Have an existing stream of this type
                Some(stream.poll())
            } else {
                // The stream exists but is of the wrong type (we'll return an empty stream in this case)
                Some(Ok(Async::Ready(None)))
            }
        } else {
            // No stream started yet
            None
        }
    }

    ///
    /// Sets the stream for reading the specified symbol
    ///
    pub fn set_stream<TStreamItem: 'static>(&mut self, symbol: FloScriptSymbol, stream: Box<dyn Stream<Item=TStreamItem, Error=()>+Send>) {
        // Store the stream in a StreamRef (this is used so we can cast it back via Any: annoyingly we end up with a box in a box here)
        let stream = StreamRef(stream);

        // Insert into the active stream
        self.active_streams.insert(symbol, Mutex::new(Box::new(stream)));
    }
}

impl Debug for DerivedStateData {
    fn fmt(&self, formatter: &mut Formatter) -> result::Result<(), fmt::Error> {
        write!(formatter, "DerivedStateData {{ namespace: <>, dependencies: {:?} }}", self.dependencies)?;

        Ok(())
    }
}

type ResolveFunction<'vm, TValue> = FunctionRef<'vm, fn(UserdataValue<DerivedStateData>) -> (UserdataValue<DerivedStateData>, TValue)>;

///
/// Monad representing a state value
/// 
/// A state value carries along with the symbols that were read from. We use this later on to decide
/// what to re-evaluate when new data arrives via an input stream.
///
#[derive(VmType)]
pub struct DerivedState<'vm, TValue> {
    // Function for resolving the value of the monad
    resolve: ResolveFunction<'vm, TValue>
}

// Gluon's codegen doesn't seem quite up to the job of dealing with a structure with a function in it, so we need to manually implement getable/pushable
impl<'vm, 'value, TValue> Getable<'vm, 'value> for DerivedState<'vm, TValue> {
    impl_getable_simple!();

    fn from_value(vm: &'vm Thread, value: Variants<'value>) -> Self {
        // Fetch the data from the value
        let data = match value.as_ref() {
            ValueRef::Data(data)    => data,
            other                   => panic!("Unexpected value while retrieving DerivedState: {:?}", other)
        };

        // Read the fields
        let resolve = data.lookup_field(vm, "resolve").expect("Cannot find the `resolve` field while retrieving DerivedState");

        // Decode into a derived state
        DerivedState {
            resolve: ResolveFunction::from_value(vm, resolve)
        }
    }
}

// Similarly, the codegen can't deal with a functionref when generating a pushable item
impl<'vm, TValue: 'static+VmType+Sized> Pushable<'vm> for DerivedState<'vm, TValue>
where TValue::Type : Sized {
    fn push(self, context: &mut ActiveThread<'vm>) -> Result<()> {
        let vm = context.thread();

        // Push the field values
        ResolveFunction::push(self.resolve, context)?;

        // Turn into a record
        let field_names = [vm.global_env().intern("resolve")?];
        context.context().push_new_record(vm, 1, &field_names)?;

        Ok(())
    }
}

///
/// Generates the flo.computed.prim extern module for a Gluon VM
///
fn load_prim(vm: &Thread) -> Result<ExternModule> {
    vm.register_type::<DerivedStateData>("flo.computed.prim.DerivedStateData", &[])?;
    vm.register_type::<DerivedState<A>>("flo.computed.prim.DerivedState", &["a"])?;

    ExternModule::new(vm, record! {
        type DerivedState a                 => DerivedState<A>,
        type DerivedStateData               => DerivedStateData
    })
}

///
/// Generates the flo.computed extern module for a Gluon VM
///
pub fn load_flo_computed(vm: &Thread) -> Result<()> {
    // Add the primitives module
    import::add_extern_module(&vm, "flo.computed.prim", load_prim);

    // And the gluon module
    let flo_computed    = include_str!("derived_state.glu");
    let mut compiler    = Compiler::default();
    compiler.load_script(vm, "flo.computed", flo_computed)
        .map_err(|err| err.emit_string(&compiler.code_map()))
        .expect("load flo.computed");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use gluon::*;
    use gluon::vm::api::*;
    use gluon::vm::primitives;

    use futures::future;
    use futures::stream;
    use futures::executor;

    #[test]
    fn make_type_from_derived_state() {
        let vm = new_vm();
        load_flo_computed(&vm).unwrap();

        // Gluon only imports user data types if the corresponding module has previously been imported
        Compiler::default().run_expr::<()>(&vm, "importfs", "import! std.fs\n()").unwrap();
        Compiler::default().run_expr::<()>(&vm, "importcomputed", "import! flo.computed\n()").unwrap();

        // IO monad
        let _some_type = IO::<i32>::make_type(&vm);

        // DirEntry is defined in the standard gluon library: it illustrates this issue does is not a bug with how DerivedState is declared
        let _some_type = UserdataValue::<primitives::DirEntry>::make_type(&vm);
        let _some_type = primitives::DirEntry::make_type(&vm);

        // Ultimate goal of this test: we should be able to get the type for DerivedState
        let _some_type = DerivedState::<i32>::make_type(&vm);
    }

    #[test]
    #[should_panic]
    fn user_data_import_issue_not_fixed() {
        // Without the import! side-effects on the VM, user data types are missing (if this test starts failing, we should be able to remove the import! above and when loading flo.computed)
        let vm = new_vm();
        let _some_type = UserdataValue::<primitives::DirEntry>::make_type(&vm);
    }

    #[test]
    fn store_and_poll_user_stream() {
        let namespace           = Arc::new(Desync::new(GluonScriptNamespace::new()));
        let mut derived_state   = DerivedStateData::new(Arc::clone(&namespace));
        let stream              = stream::iter_ok::<_, ()>(vec![1, 2, 3]);
        let symbol              = FloScriptSymbol::new();

        derived_state.set_stream(symbol, Box::new(stream));

        let first_symbol        = future::poll_fn(move || derived_state.poll_stream::<i32>(symbol).unwrap());
        let mut first_symbol    = executor::spawn(first_symbol);

        let first_symbol        = first_symbol.wait_future().unwrap();

        assert!(first_symbol == Some(1));
    }

    #[test]
    fn wrong_type_produces_empty_stream() {
        let namespace           = Arc::new(Desync::new(GluonScriptNamespace::new()));
        let mut derived_state   = DerivedStateData::new(Arc::clone(&namespace));
        let stream              = stream::iter_ok::<_, ()>(vec![1, 2, 3]);
        let symbol              = FloScriptSymbol::new();

        derived_state.set_stream(symbol, Box::new(stream));

        let first_symbol        = future::poll_fn(move || derived_state.poll_stream::<f32>(symbol).unwrap());
        let mut first_symbol    = executor::spawn(first_symbol);

        let first_symbol        = first_symbol.wait_future().unwrap();

        assert!(first_symbol == None);
    }
}
