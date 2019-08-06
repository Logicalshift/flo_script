use std::marker::PhantomData;

///
/// Reads from an input stream
///
pub struct InputStream<SymbolType> {
    symbol: PhantomData<SymbolType>
}
