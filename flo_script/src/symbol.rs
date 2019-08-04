use std::sync::*;
use std::collections::HashMap;
use std::sync::atomic::{Ordering, AtomicU64};

lazy_static! {
    static ref NEXT_SYMBOL:     AtomicU64 = AtomicU64::new(0);
    static ref NAME_TO_SYMBOL:  Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
    static ref SYMBOL_TO_NAME:  Mutex<HashMap<u64, String>> = Mutex::new(HashMap::new());
}

///
/// An abstract representation of a symbol in a script
///
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FloScriptSymbol {
    /// The ID set for this symbol
    id: u64
}

///
/// Assigns a new symbol ID
///
fn assign_symbol_id() -> u64 {
    NEXT_SYMBOL.fetch_add(1, Ordering::Relaxed)
}

impl FloScriptSymbol {
    ///
    /// Creates a new symbol (with no name, so it cannot be referenced within a script)
    ///
    pub fn new() -> FloScriptSymbol {
        let symbol_id = assign_symbol_id();
        FloScriptSymbol {
            id: symbol_id
        }
    }

    ///
    /// Retrieves the symbol with the specified name
    ///
    pub fn with_name(name: &str) -> FloScriptSymbol {
        let mut name_to_symbol = NAME_TO_SYMBOL.lock().unwrap();

        if let Some(existing_id) = name_to_symbol.get(name) {
            // This name is already in use
            FloScriptSymbol {
                id: *existing_id
            }
        } else {
            // Assign a new symbol ID for the name
            let mut symbol_to_name  = SYMBOL_TO_NAME.lock().unwrap();
            let symbol_id           = assign_symbol_id();

            // Store so future requests retrieve this name
            name_to_symbol.insert(name.to_string(), symbol_id);
            symbol_to_name.insert(symbol_id, name.to_string());

            // Return this symbol
            FloScriptSymbol {
                id: symbol_id
            }
        }
    }

    ///
    /// Retrieves the name of this symbol, if it's a named symbol
    ///
    pub fn name(&self) -> Option<String> {
        let symbol_to_name = SYMBOL_TO_NAME.lock().unwrap();
        symbol_to_name.get(&self.id).cloned()
    }
}
