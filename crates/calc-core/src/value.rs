#[derive(Debug, PartialEq)]
pub struct Value {
    pub number: f64,
}

impl Value {
    pub(crate) fn number(number: f64) -> Self {
        Self { number }
    }
}
