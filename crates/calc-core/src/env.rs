use std::collections::HashMap;

use crate::{InternalQualifiedName, Symbol, Value};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<InternalQualifiedName, Value>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, name: &InternalQualifiedName) -> Option<&Value> {
        self.values.get(name)
    }

    pub(crate) fn set(&mut self, name: InternalQualifiedName, value: Value) {
        self.values.insert(name, value);
    }

    pub(crate) fn resolve(
        &self,
        name: &InternalQualifiedName,
        scope: &[Symbol],
    ) -> Option<InternalQualifiedName> {
        if name.parts.len() > 1 {
            return self.values.contains_key(name).then(|| name.clone());
        }

        let symbol = *name.parts.first()?;
        for len in (0..=scope.len()).rev() {
            let mut parts = scope[..len].to_vec();
            parts.push(symbol);
            let qualified = InternalQualifiedName::new(parts);
            if self.values.contains_key(&qualified) {
                return Some(qualified);
            }
        }

        None
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
