# Zed Improvements 001
## Purpose
- To set the default indent in the Zed editor of calc files to 2 spaces
- The autocomplete doesn't overwrite the start of the variable path, e.g.:
```yaml
level1:
  level2:
    var = 0

# Autocomplete can end up like this if it stops at `level1.` and then shows the autocomplete drop down
level1.level1.level2.var
```
- Turn of auto-complete options in comments. Only make auto complete suggestions on the right hand side of an assignment`
## Existing Code
- Does not specify the number of spaces for indents that Zed should use by default
- The current code adds the full path selected to whatever part of the code has already been typed
- Auto-complete makes suggestions for completion in comments
## Implementation Plan
- Update `crates/calc-zed/languages/calc/config.toml` to set Calc editor indentation defaults to 2 spaces.
- Add or update LSP completion tests for qualified variable completion after a partially typed path.
- Change completion items for qualified variable references so accepting a completion replaces the already typed path prefix instead of appending the full path to it.
- Determine the completion context from the current line and cursor position before returning suggestions.
- Return no completion items when the cursor is inside a comment.
- Return no completion items unless the cursor is on the right hand side of an assignment.
- Keep completion behavior for visible variables and qualified references consistent with the language rules in `core.md` and `lsp.md`.
- Rebuild and verify with:
```sh
cargo fmt --all --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build -p calc-zed --target wasm32-wasip1
```
## Local Zed Dev Extension Rebuild
- Reinstall the local language server binary:
```sh
cargo install --path crates/calc-lsp
```
- Rebuild the Zed extension:
```sh
cargo build -p calc-zed --target wasm32-wasip1
```
- In Zed, use the installed Calc dev extension's `Rebuild` button, or reinstall the dev extension from:
```text
/Users/david/src/tries/2026-05-17-calc/calc/crates/calc-zed
```
- Restart Zed if it keeps using an old `calc-lsp` process.
