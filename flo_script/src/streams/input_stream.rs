use std::marker::PhantomData;

use futures::*;

///
/// Reads from an input stream
///
pub struct InputStream<Symbol> {
    symbol: PhantomData<Symbol>
}

impl<Symbol: Clone> Stream for InputStream<Symbol> {
    type Item   = Symbol;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Symbol>, ()> {
        unimplemented!("Input stream poll")
    }
}
