# Calc core
## Purpose
- a calculation engine for calc, that is independent of the editor or the file system
- is able to incrementally reevaluate a source file
## Design
- it is built in rust
- Structs and enums do not derive clone except in exceptional circumstances
- Tokens and AST nodes use `Symbol` handles into a string interner rather than storing identifier strings.
- Token, Lexer, Expr, Statement, Parser, Environment, and Evaluate Statement are internal implementation details.
- The editor-facing public API is `evaluate_new_document`, `evaluate_edited_document`, `DocumentEvaluation`, `LineEvaluation`, `Value`, `Symbol`, `QualifiedName`, `Span`, `CalcError`, and `CalcErrorKind`.
### Incremental document evaluation design
- A new document evaluation returns a reusable document state
- An edited document evaluation takes previous document state and the new source text
- It identifies the changed line range by comparing old and new line text
- It reparses and reevaluates changed lines
- It reevaluates later lines that depend on variables defined by changed lines
- It preserves prior line evaluations for unchanged lines that do not depend on changed variables
- Inserted and deleted lines are part of the changed line range.
- Unchanged suffix lines may be preserved with updated line numbers when they do not depend on changed variables.
- If a changed line defines a variable, later lines depending on that variable are reevaluated until dependency propagation is complete.
## Interner
- A string interner to avoid copying strings
- The interner is internal to `DocumentEvaluation`.
- Public callers access interned text through `DocumentEvaluation::symbol_text`.
## Symbol
- A handle to a string in the interner
- Callers cannot construct arbitrary symbols
- public struct:
```rust
pub struct Symbol;
```
## Span
- Defines source position
- Spans use zero-based byte offsets.
- Spans are relative to the line source, not the whole document.
- `end` is exclusive.
- public struct:
```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```
## Token (internal)
- Defines the tokens that the lexer recognizes
## Lexer (internal)
- Turns source text into tokens
## Expr (internal)
- An expression tree
## Statement (internal)
- A statement is an expression statement or an assignment statement
## Parser (internal)
- A function that turns tokens into an AST with a statement as the top node
## Environment (internal)
- Stores variables
## Evaluate Statement (internal)
- Evaluates a parsed statement
## Value
- Represents a typed value
- public enum:
```rust
pub enum Value {
    Number(f64),
    Currency(String),
    Money { currency: String, minor_units: i64 },
    Text(String),
    Boolean(bool),
    List(Vec<Value>),
}

impl Value {
    pub fn display_text(&self) -> String;
}
```
- `Money.minor_units` stores the value rounded to 2 decimal places.
- Currency strings use ISO 4217 currency codes.
- `Value` may derive `Clone` because variable lookup returns stored non-copy values.
## Evaluate New Document
- evaluates a document for the first time
- public API:
```rust
pub fn evaluate_new_document(source: &str) -> DocumentEvaluation;
```
## Evaluate Edited Document
- evaluates a document that has been evaluated at least once
- consumes the previous `DocumentEvaluation`
- public API:
```rust
pub fn evaluate_edited_document(
    previous: DocumentEvaluation,
    source: &str,
) -> DocumentEvaluation;
```
## Document Evaluation
- the result of evaluating a document
- public struct:
```rust
pub struct DocumentEvaluation {
    pub lines: Vec<LineEvaluation>,
}

impl DocumentEvaluation {
    pub fn symbol_text(&self, symbol: Symbol) -> &str;
}
```
## Line Evaluation
- the result of evaluating a line
- `line` is zero based
- blank lines are included and return `Ok(None)`
- `defines` is the variable assigned to the line, if any
- public struct:
```rust
pub struct LineEvaluation {
    pub line: usize,
    pub result: Result<Option<Value>, CalcError>,
    pub defines: Option<QualifiedName>
}
```
## Qualified Name
- `parts` always contains the full resolved variable path
- for top-level variables, `parts` contains one symbol.
- for section variables, `parts` constains section path symbols followed by the variable symbol
- public struct:
```rust
pub struct QualifiedName {
    pub parts: Vec<Symbol>
}
 ```
## Calc Error
- reports a calculation error
```rust
pub struct CalcError {
    pub kind: CalcErrorKind,
    pub span: Span,
}
```
## Calc Error Kind
- the type of error being reported
```rust
pub enum CalcErrorKind {
    UnexpectedCharacter,
    InvalidNumber,
    ExpectedExpression,
    ExpectedToken,
    UnexpectedToken,
    UndefinedVariable,
    DivisionByZero,
    InvalidIndentation,
    InvalidSectionHeader,
    UnsupportedTypeOperation,
    MixedListTypes,
}
```
## Expression language
### Numbers
- Supports decimal numbers: `1`, `1.5`, `.5`, `5.`
- Does not support scientific notation initially
### Currency
- Supports currency values from the ISO 4217 list, such as `GBP`, `USD`, and `EUR`.
- Supports currency symbols `£`, `$`, and `€` as aliases for `GBP`, `USD`, and `EUR`.
- Bare ISO 4217 codes and supported currency symbols are currency literals.
### Money
- Supports money literals made from a currency immediately followed by a number.
- Examples: `£100`, `USD99.99`.
- Money values are stored rounded to 2 decimal places.
### Text
- Supports text literals between double quotes.
- Text literals do not support escape syntax.
### Boolean
- Supports boolean literals `true` and `false`.
### Lists
- Supports non-empty list literals between `[` and `]`.
- List items are separated by commas.
- The final item may optionally be followed by a comma.
- Lists must contain values of the same inferred type.
### Operators
- Supports `+`, `-`, `*`, `/`
- Supports unary minus
- `Number +/- Number -> Number`
- `Money<Currency> +/- Money<Currency> -> Money<Currency>` when both money values have the same currency.
- `Text + Text -> Text`
- `Number *// Number -> Number`
- `Money<Currency> *// Number -> Money<Currency>`
- `-Number -> Number`
- `-Money<Currency> -> Money<Currency>`
- Any other operator and type combination is invalid.
- Operator precedence:
  - parentheses
  - unary minus
  - multiplication/division
  - addition/subtraction
- Operators are left-associative except unary minus
### Variables
- Variable names start with a letter or `_`
- Variable names may contain letters, digits, and `_`
- Variables are case-sensitive
### Statements
- Expression statement: `1 + 2`
- Assignment statement: `name = expression`
- Assignment stores the evaluated value for later lines
- Blank lines are valid and evaluate to no value.
- Spaces, tabs, and trailing whitespace are ignored.
### Sections
- A section header is `name:`.
- Indentation determines section nesting.
- Indentation is measured in leading spaces.
- Tabs are not allowed for indentation.
- A statement belongs to the nearest preceding section header with lower indentation.
- A section header defines a namespace, not a value.
- Assignments inside a section define qualified variables.
- Unqualified references resolve first in the current section, then parent sections, then top level.
- Qualified references use dot syntax, such as `house.stairs.total`.
- Section headers evaluate to no value.
- Section headers contain a single unqualified name.
- Dotted section headers are not supported.
- Any number of leading spaces may be used for indentation.
- Tabs in leading indentation are invalid.
- A section header at the same indentation as the current section starts a sibling section.
- A section header at lower indentation dedents to the nearest parent section with lower indentation.
- A statement at lower indentation dedents to the nearest parent section with lower indentation.
- Section headers may have trailing comments.
- `InvalidIndentation` is reported for tabs in leading indentation.
- `InvalidSectionHeader` is reported for malformed section headers, including dotted section headers.
## Comments
- `#` starts a comment outside expressions.
- A comment continues to the end of the line.
- Comments are ignored during parsing and evaluation.
- A line containing only whitespace and a comment is a blank line.
## Result comments
- A result comment is a trailing comment whose first non-whitespace text is `=`.
- Result comments are ignored during parsing and evaluation like other comments.
- Result comments do not define values and do not affect dependencies.
## Boundary
- `calc-core` owns calculation semantics.
- `tree-sitter-calc` owns syntax recognition.
- `calc-lsp` owns LSP behavior.
- `calc-zed` owns Zed extension registration and language server startup.
