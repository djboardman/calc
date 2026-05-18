use std::collections::HashMap;

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct Symbol(usize);

#[derive(Debug)]
pub struct StringInterner {
    strings: Vec<String>,
    symbols: HashMap<String, Symbol>,
}

impl StringInterner {
    pub(crate) fn new() -> Self {
        Self {
            strings: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub(crate) fn intern(&mut self, value: &str) -> Symbol {
        if let Some(symbol) = self.symbols.get(value) {
            return *symbol;
        }

        let symbol = Symbol(self.strings.len());
        self.strings.push(value.to_string());
        self.symbols.insert(value.to_string(), symbol);
        symbol
    }

    pub(crate) fn get(&self, symbol: Symbol) -> &str {
        &self.strings[symbol.0]
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
