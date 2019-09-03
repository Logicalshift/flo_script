use super::error::*;
use super::symbol::*;
use super::gluon_host::derived_state::*;

use gluon::{Compiler};
use gluon::vm;
use gluon::vm::thread::{RootedThread, Thread};
use gluon::vm::ExternModule;
use gluon::vm::api::{UserdataValue, Function, FunctionRef, Primitive, VmType, Pushable, Getable, FutureResult};
use futures::*;
use futures::sync::oneshot;

use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::{Debug};
use std::result;
use std::sync::*;

///
/// Provides a description for a type that can be used when streaming from a script
///
#[derive(Clone)]
pub struct ScriptTypeDescription {
    /// The ID of this type so it can be compared to others
    type_id: TypeId,

    /// Creates an extern module loader for the 'resolve' function of a derived state of this type
    derived_state_resolve: Arc<dyn Fn(FloScriptSymbol) -> Box<dyn FnMut(&Thread) -> vm::Result<ExternModule> + Send + 'static>+Send+Sync>
}

impl ScriptTypeDescription {
    ///
    /// True if this script type matches the specified type
    ///
    pub fn is<T: 'static+ScriptType>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

impl PartialEq for ScriptTypeDescription {
    fn eq(&self, compare_to: &ScriptTypeDescription) -> bool {
        self.type_id.eq(&compare_to.type_id)
    }
}

impl Debug for ScriptTypeDescription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.type_id)
    }
}

///
/// Trait implemented by things that can be used with a script
///
pub trait ScriptType : Any+Clone+Send {
    ///
    /// Creates or retrieves a description for this type
    ///
    fn description() -> ScriptTypeDescription;
}

impl<T> ScriptType for T 
where   for<'vm, 'value> T: Any+VmType+Getable<'vm, 'value>+Pushable<'vm>+Sized+Clone+Send,
        T::Type : Sized {
    fn description() -> ScriptTypeDescription {
        let type_id                 = TypeId::of::<T>();
        let state_resolver          = userdata_derived_state_resolve::<T>;
        let derived_state_resolve   = Arc::new(move |symbol: FloScriptSymbol| {
            // Can't pass symbols directly to gluon at the moment, so get the ID
            let symbol_id = symbol.id();

            let fun: Box<dyn FnMut(&Thread) -> vm::Result<ExternModule> + Send + 'static> = Box::new(move |thread| {
                let mut compiler = Compiler::default();

                let state_resolver = primitive!(2, state_resolver);
                let resolve_fn: FunctionRef<fn(Primitive<fn(u64, UserdataValue<DerivedStateData>) -> (UserdataValue<DerivedStateData>, T)>, u64) -> Function<RootedThread, fn(UserdataValue<DerivedStateData>) -> (UserdataValue<DerivedStateData>, T)> > = compiler.run_expr(thread, "foo", r#"\resolve symbol_id -> resolve symbol_id"#).unwrap().0;

                // let resolve = resolve_fn.call(state_resolver, symbol_id);

                ExternModule::new(thread, primitive!(0, || { 0 }))
            });
            fun
        });

        ScriptTypeDescription {
            type_id,
            derived_state_resolve
        }
    }
}

//
// Gluon-specific things
// 
// (I'd really like to move these elsewhere so we can support multiple scripting languages more easily but for now we'll limit ourselves
// to Gluon as I'm not sure there's any way of doing this without specialization of some kind)
// 

///
/// Variant of derived_state_resolve that uses the gluon UserdataValue struct
///
fn userdata_derived_state_resolve<Symbol: 'static+ScriptType>(symbol_id: u64, state_data: UserdataValue<DerivedStateData>) -> FutureResult<impl Future<Item=(UserdataValue<DerivedStateData>, Symbol), Error=vm::Error>+Send>
where   Symbol:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
<Symbol as VmType>::Type:   Sized {
    let symbol                      = FloScriptSymbol::with_id(symbol_id);
    let UserdataValue(state_data)   = state_data;

    let resolved                    = derived_state_resolve(symbol, state_data);
    let resolved                    = resolved.map(|(state, symbol)| (UserdataValue(state), symbol));
    let resolved                    = resolved.map_err(|_| vm::Error::Dead);

    FutureResult(resolved)
}

///
/// Creates the 'resolve' function for the DerivedState for a symbol a namespace
///
fn derived_state_resolve<Symbol: 'static+ScriptType>(symbol: FloScriptSymbol, state_data: DerivedStateData) -> impl Future<Item=(DerivedStateData, Symbol), Error=()>+Send
where   Symbol:             for<'vm, 'value> Getable<'vm, 'value> + VmType + Send + 'static,
<Symbol as VmType>::Type:   Sized {
    // Poll for the stream if it's not available
    let mut future_stream   = if !state_data.has_stream(symbol)  {
        let namespace       = state_data.get_namespace();
        let future_stream   = namespace.future(move |namespace| namespace.read_state_stream::<Symbol>(symbol));
        let future_stream: Box<dyn Future<Item=FloScriptResult<Box<dyn Stream<Item=Symbol, Error=()>+Send>>, Error=oneshot::Canceled>+Send> = Box::new(future_stream); // here is a place Rust's type inference lets us down :-(
        Some(future_stream)
    } else {
        None
    };

    // We own the state data until we return a result
    let mut state_data      = Some(state_data);

    future::poll_fn(move || {
        let current_state = state_data.as_mut().unwrap();

        loop {
            if let Some(actual_future_stream) = future_stream.as_mut() {

                // Trying to retrieve the stream: poll that first
                match actual_future_stream.poll() {
                    Ok(Async::NotReady)             => { return Ok(Async::NotReady); },
                    Err(_)                          => { return Err(()); },
                    Ok(Async::Ready(Err(_)))        => { return Err(()); },
                    Ok(Async::Ready(Ok(stream)))    => {
                        // Stream retrieved: set it and start again
                        current_state.set_stream(symbol, stream);
                        future_stream = None;
                    }
                }

            } else if let Some(result) = current_state.poll_stream::<Symbol>(symbol) {

                // The stream is currently active for this symbol
                return match result {
                    Ok(Async::Ready(Some(result)))  => Ok(Async::Ready((state_data.take().unwrap(), result))),
                    Ok(Async::Ready(None))          => Ok(Async::NotReady),
                    Ok(Async::NotReady)             => Ok(Async::NotReady),
                    Err(err)                        => Err(err)
                };

            } else {

                // The stream is not active for this symbol: start polling for it (again)
                let namespace   = current_state.get_namespace();
                future_stream   = Some(Box::new(namespace.future(move |namespace| namespace.read_state_stream::<Symbol>(symbol))));

            }
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_i32_type_description() {
        let _t: ScriptTypeDescription = i32::description();
    }
}