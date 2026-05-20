use crate::Symbol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    pub parts: Vec<Symbol>,
}

impl QualifiedName {
    pub(crate) fn new(parts: Vec<Symbol>) -> Self {
        Self { parts }
    }
}
