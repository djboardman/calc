# Calc LSP crate
## Purpose
- This crate `calc-lsp` provides Language Server Protocol support for Calc.
- It connects editor document events to `calc-core`.
- It owns completion and diagnostic behavior for Calc editor integrations.
- It is independent of any specific editor.
## Design
- Built in `rust`.
- Depends on `calc-core`.
- Implements an LSP server.
- Does not implement calculator parsing or evaluation.
- Uses `calc-core` for document evaluation.
- Maintains evaluated state for open documents.
## Public API
- Exposes a language server executable named `calc-lsp`.
- The executable reads LSP messages from stdin.
- The executable writes LSP messages to stdout.
## Internal Components
### Server
- Owns the LSP connection.
- Handles LSP requests and notifications.
- Dispatches document events to the document store.
### Document Store
- Tracks open documents by URI.
- Stores the latest source text for each document.
- Stores the latest `calc-core::DocumentEvaluation` for each document.
- Uses `evaluate_new_document` for newly opened documents.
- Uses `evaluate_edited_document` for changed documents.
### Document Input Adapter
- Converts LSP document text into string input expected by `calc-core`.
- Applies LSP text changes to stored source text.
- Detects whether a document is new or previously evaluated.
### Completion Provider
- Handles `textDocument/completion`.
- Uses the latest evaluated document state.
- Provides previously declared variable names as completion items.
- Does not parse or evaluate expressions directly.
### Diagnostics Provider
- Publishes diagnostics after document open and document change.
- Converts `calc-core::CalcError` values into LSP diagnostics.
- Converts line-relative `CalcError.span` values into LSP ranges.
- Clears diagnostics when a document has no errors.
### Result Adapter
- Converts `calc-core::LineEvaluation` values into LSP-facing output.
- Resolves symbols through `DocumentEvaluation::symbol_text`.
- Does not evaluate calculations itself.
### Configuration
- Provides default server settings.
- Reads initialization options if settings are added later.
- Does not affect calculation semantics owned by `calc-core`.
## LSP Capabilities
### Initialize
- Responds to `initialize`.
- Advertises text document synchronization.
- Advertises completion support.
### Text Document Synchronization
- Handles `textDocument/didOpen`.
- Handles `textDocument/didChange`.
- Handles `textDocument/didClose`.
### Completion
- Handles `textDocument/completion`.
- Completion items suggest variables declared before the current line.
- Completion items do not include calculated values.
- Completion items insert the selected variable name.
### Diagnostics
- Publishes diagnostics for calculation errors.
- Diagnostics use line-relative spans from `calc-core` converted to LSP ranges.
- Diagnostics include stale result comment warnings.
- Diagnostics are cleared on `textDocument/didClose`.
## Document Behavior
- Each opened document is evaluated with `evaluate_new_document`.
- Each changed document is evaluated with `evaluate_edited_document`.
- The server keeps the latest document source and evaluation state.
- Closed documents are removed from the document store.
### Inlay Hints
- Does not advertise inlay hint support for calculation results.
- Calculation results are written with result comments.
## Comments
- Comments do not produce diagnostics.
- Completions ignore comment text.
## Result comments
- The language server advertises document formatting support.
- Formatting a Calc document writes evaluated results into the document as result comments.
- Result comments use the format `# = value`.
- Assignment lines whose value is a numeric literal do not receive result comments.
- Writing result comments replaces an existing trailing result comment on the same line, including literals with a unary operator.
- Writing result comments does not replace ordinary trailing comments.
- Result comments are written only for successfully evaluated non-blank lines.
- Lines with errors or no value do not receive result comments.
- Inlay hints are not used for calculation results when result comment writing is enabled.
- If a result comment exists but does not match the current evaluated result, publish a warning diagnostic on the result comment.
- Missing result comments do not produce diagnostics.
## Sections
- Section headers do not receive result comments.
- Section headers do not produce diagnostics unless malformed.
- Completion includes variables visible from the current section.
- Completion supports qualified variable references.
- Completion for visible variables inserts the shortest valid name from the current section.
- Completion for qualified references inserts the full qualified name.
## Boundary
- `calc-core` owns calculation semantics.
- `calc-lsp` owns LSP behavior.
- `tree-sitter-calc` owns syntax recognition.
- `calc-zed` owns Zed extension registration and language server startup.
- `calc-lsp` must not duplicate parser, evaluator, or document evaluation logic from `calc-core`.
