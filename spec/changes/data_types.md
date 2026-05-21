# Data Types
## Purpose
- To introduce data types to calc
- To introduce type inference
- To introduce casting between types
## Existing Functionality
- Currently all values in calc are a single type of Value
## New Functionality
- Introduce a set of basic types:
  - Number
  - Currency
  - Money<Currency>
  - Text
  - Boolean
  - List<Type>
- Use type inference rules to recognize a type from the assigned value or outcome of operations
  - Number:
    - Digits without or without a decimal point
    - e.g. `12` or `12.24`
  - Currency from the ISO 4217 list:
    - Any ISO 4217 currency code is supported.
    - Supported currency symbols are `£`, `$`, `€` which are aliases for `GBP`, `USD`, `EUR`.
  - Money
    - A currency type followed by a number
    - The number is stored as a decimal with 2 decimal places whether or not there is a decimal place or the number of decimal places
    - e.g. `£100` or `USD99.99`
  - Text
    - Whatever is between begin and end double quotes `"`
    - e.g. `"Hello, world!"`
  - Boolean
    - The literal values `true` or `false`
  - List<Type>
    - A comma separated list between `[` and `]` with or without a final `,` of the same type Type
    - e.g. `[1, 2, 3]` or `[1, 2, 3,]
- Define the type of basic operators (any other combinations are invalid)
  - Binary operators:
    - `Number +/- Number -> Number`
    - `Money<Currency> +/- Money<Currency> -> Money<Currency>`
    - `Text + Text -> Text`
    - `Number *// Number -> Number`
    - `Money<Currency> *// Number -> Money<Currency>`
  - Unary operators:
    - `-Number -> Number`
    - `-Money<Currency> -> Money<Currency>`
## Implementation Plan
- Update `spec/core.md` after this change is accepted so the public `Value` API can represent all supported data types instead of only `number: f64`.
- Add internal value variants for `Number`, `Currency`, `Money<Currency>`, `Text`, `Boolean`, and `List<Type>`.
- Add a decimal representation for money values so money is stored with exactly 2 decimal places.
- Add lexer support for:
  - Any ISO 4217 currency code when used as a currency prefix for money literals.
  - Currency symbols `£`, `$`, and `€` when used as currency prefixes for money literals.
  - Double quoted text literals.
  - Boolean literals `true` and `false`.
  - List delimiters `[` and `]`.
  - Commas inside lists.
- Add parser support for:
  - Money literals.
  - Text literals.
  - Boolean literals.
  - List literals with an optional trailing comma.
  - Lists containing only values of the same inferred type.
- Add evaluator support for the specified operator type rules.
- Add errors for unsupported operator/type combinations and mixed-type lists.
- Preserve existing section, qualified name, comment, result comment, formatting, and completion behavior.
- Update result comment formatting to render non-number values in a stable textual form.
- Update stale result comment diagnostics to compare against the rendered value for every supported type.
- Update `spec/tree-sitter.md` after this change is accepted to include the new literal syntax.
- Update the external `tree-sitter-calc` grammar for the new literals and list syntax.
- Regenerate, commit, and push the external `tree-sitter-calc` parser, then update `crates/calc-zed/extension.toml` with the new grammar `rev`.
## Test Titles
- `core infers number values from decimal literals`
- `core infers money values from currency symbol literals`
- `core infers money values from ISO currency literals`
- `core stores money values with two decimal places`
- `core infers text values from double quoted literals`
- `core infers boolean values from true and false literals`
- `core infers list values from homogeneous list literals`
- `core accepts list literals with a trailing comma`
- `core rejects mixed type list literals`
- `core evaluates number arithmetic with existing operators`
- `core evaluates same-currency money addition and subtraction`
- `core rejects mixed-currency money addition and subtraction`
- `core evaluates text concatenation with plus`
- `core evaluates money multiplied by number`
- `core evaluates money divided by number`
- `core rejects unsupported operator type combinations`
- `core evaluates unary minus for number and money`
- `lsp formats result comments for all supported value types`
- `lsp warns for stale result comments for non-number values`
- `tree-sitter parses money text boolean and list literals`
## Local Zed Dev Extension Rebuild
- Reinstall the local language server binary:
```sh
cargo install --path crates/calc-lsp
```
- Rebuild the Zed extension:
```sh
cargo build -p calc-zed --target wasm32-wasip1
```
- If the external Tree-sitter grammar changes, regenerate and push it:
```sh
cd ../tree-sitter-calc
npx tree-sitter-cli generate
git add grammar.js tree-sitter.json src
git commit -m "Update calc grammar"
git push
git rev-parse HEAD
```
- Update `crates/calc-zed/extension.toml` so `[grammars.calc].rev` matches the pushed grammar commit.
- Remove stale local grammar checkouts if needed:
```sh
rm -rf crates/calc-zed/grammars
```
- In Zed, use the installed Calc dev extension's `Rebuild` button, or reinstall the dev extension from:
```text
/Users/david/src/tries/2026-05-17-calc/calc/crates/calc-zed
```
- Restart Zed if it keeps using an old `calc-lsp` process.
