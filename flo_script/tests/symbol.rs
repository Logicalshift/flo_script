use flo_script::*;

#[test]
fn new_symbols_are_different() {
    let symbol1 = FloScriptSymbol::new();
    let symbol2 = FloScriptSymbol::new();

    assert!(symbol1 != symbol2);
}

#[test]
fn same_name_is_same_symbol() {
    let symbol1 = FloScriptSymbol::with_name("Hello");
    let symbol2 = FloScriptSymbol::with_name("Hello");

    assert!(symbol1 == symbol2);
}

#[test]
fn different_name_is_different_symbol() {
    let symbol1 = FloScriptSymbol::with_name("Hello");
    let symbol2 = FloScriptSymbol::with_name("World");

    assert!(symbol1 != symbol2);
}

#[test]
fn retrieve_symbol_name() {
    let symbol1 = FloScriptSymbol::with_name("Hello");

    assert!(symbol1.name().is_some());
    assert!(symbol1.name() == Some("Hello".to_string()));
}

#[test]
fn anonymous_symbols_have_no_name() {
    let symbol1 = FloScriptSymbol::new();

    assert!(symbol1.name().is_none());
}
