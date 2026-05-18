#[derive(Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) fn source<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}
