# Calc zed crate
## Purpose
- This crate `calc-zed` integrates Calc with Zed.
- Starts the `calc-lsp` language server for Zed.
- Keeps Zed-specific behavior separate from the editor-independent calculation engine and editor-independent language server.
## Design
- Built in `rust`
- Uses `zed_extension_api` crate
- Depends on `calc-lsp`
- References the `tree-sitter-calc` grammar
- Does not implement calculator parsing or evaluation
- Does not implement document synchronization, completions, or diagnostics
- Zed editor text, completions, and diagnostics are handled by `calc-lsp` through LSP
## Public API
- Only exposes the Zed extension entry point
- The extension type `CalcZed` implements `zed_extension_api::Extension`
## Zed extension entry point
```rust
zed::register_extension!(CalcZed);
```
## Language server command
- Language server id is `calc`
- `CalcZed` implements `language_server_command`
- `language_server_command` returns the command Zed uses to start `calc-lsp`
- If the requested language server id is not `calc`, return an error
```rust
fn language_server_command(
    &mut self,
    language_server_id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> zed::Result<zed::Command>;
```
## Language server binary
- The language server executable is named `calc-lsp`
- `calc-zed` must not assume the opened Zed worktree is the Calc source workspace
- Packaged releases locate bundled `calc-lsp` binaries under `bin`
- Language server lookup order:
  - bundled binary under `bin`
  - `calc-lsp` on `PATH`
- If neither location contains `calc-lsp`, return an error explaining that `calc-lsp` must be built and installed or bundled
## Language activation
- Calc support activates for files with the `.calc` extension
- Calc language recognition is provided by `tree-sitter-calc`
## Development layout
- Dev extension directory is `crates/calc-zed`
- Workspace root is two directories above the dev extension directory
- Grammar repository location is read from `crates/calc-zed/extension.toml`
## Packaged layout
- Packaged extension contains `extension.toml`
- Packaged extension contains `languages/calc/config.toml`
- Packaged extension contains bundled `calc-lsp` binaries under `bin`, if binaries are bundled
## Internal Components
### Extension
- Registers the extension with Zed
- Owns top level Zed integration behaviour
- Starts `calc-lsp` through `language_server_command`
### Language server locator
- Finds the `calc-lsp` executable
- Handles development and packaged extension layouts
- Uses the language server lookup order from this specification
- Returns a `zed::Command`
### Grammar configuration
- References the `tree-sitter-calc` grammar for `.calc` files
- Uses the external `tree-sitter-calc` repository URL configured in `crates/calc-zed/extension.toml`
- Grammar configuration includes `rev` pinned to a commit in `tree-sitter-calc`
- After changing the grammar repository or `rev`, stale local grammar checkouts under `crates/calc-zed/grammars` may be removed before reinstalling the dev extension
- Does not implement grammar behavior itself
### Configuration
- Reads Calc-related Zed settings, if settings are added
- Provides defaults when settings are absent
- Does not affect calculation semantics owned by `calc-core`
- Does not affect LSP semantics owned by `calc-lsp`
### Editor defaults
- Calc files use a default indentation width of 2 spaces.
- Calc files use spaces for indentation, not hard tabs.
### Highlighting
- `number` nodes are captured as `@number`.
- `identifier` nodes are captured as `@variable`.
- Operator tokens `+`, `-`, `*`, `/`, and `=` are captured as `@operator`.
- `comment` nodes are captured as `@comment`.
- `result_comment` nodes are captured as `@hint`.
- `section_header` names are captured as `@namespace`.
- Exact colors are controlled by the active Zed theme.
## Boundary
- `calc-core` owns calculation semantics
- `calc-lsp` owns LSP behavior
- `tree-sitter-calc` owns syntax recognition
- `calc-zed` owns Zed extension registration and language server startup
- `calc-zed` must not duplicate parser, evaluator, document evaluation, completion, diagnostic, or grammar logic from `calc-core`, `calc-lsp`, or `tree-sitter-calc`
