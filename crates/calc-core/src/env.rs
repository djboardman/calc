use std::collections::HashMap;

use crate::{Symbol, Value};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<Symbol, Value>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, name: Symbol) -> Option<&Value> {
        self.values.get(&name)
    }

    pub(crate) fn set(&mut self, name: Symbol, value: Value) {
        self.values.insert(name, value);
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
