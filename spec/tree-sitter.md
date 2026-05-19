# Calc tree-sitter grammar
## Purpose
- This package `tree-sitter-calc` provides a tree-sitter grammar for Calc source files.
- It enables Zed to recognize `.calc` files as a Calc language.
- It provides syntax structure for editor features such as highlighting.
## Design
- Built as a tree-sitter grammar package.
- Does not implement calculator evaluation.
- Does not depend on `calc-core`, `calc-lsp`, or `calc-zed`.
- Grammar syntax matches the expression language specified in `core.md`.
## Package layout
- The grammar package lives in the external `tree-sitter-calc` repository.
- `tree-sitter-calc` is a Git repository.
- The grammar definition is `tree-sitter-calc/grammar.js`.
- The tree-sitter metadata file is `tree-sitter-calc/tree-sitter.json`.
- Generated parser files live under `tree-sitter-calc/src`.
## Public API
- Exposes the tree-sitter language generated from `grammar.js`.
- Package name is `tree-sitter-calc`.
- Grammar name is `calc`.
## Grammar rules
### Source file
- A source file is zero or more lines.
- A line is a blank line, expression statement, assignment statement, comment, or statement with a trailing comment.
### Statements
- Expression statement: `1 + 2`
- Assignment statement: `name = expression`
### Expressions
- Supports decimal numbers: `1`, `1.5`, `.5`, `5.`
- Supports variables.
- Supports parentheses.
- Supports unary minus.
- Supports binary operators `+`, `-`, `*`, `/`.
- Operator precedence matches `core.md`.
### Variables
- Variable names start with a letter or `_`.
- Variable names may contain letters, digits, and `_`.
- Variables are case-sensitive.
### Whitespace
- Spaces and tabs are ignored between tokens.
- Newlines separate statements.
### Comments
- Supports line comments starting with `#`.
- Comments continue to the end of the line.
- Comments may appear after statements or on otherwise blank lines.
- Comments are distinct syntax nodes.
- Comment node name is `comment`
### Result comments
- Result comments are syntax comments.
- A result comment starts with `#` followed by optional whitespace and `=>`.
- Result comments may be highlighted distinctly from ordinary comments if supported.
- Result comments are distinct syntax nodes.
- Result node name is `result_comment`
## Zed integration
- `calc-zed` references this grammar for `.calc` files.
- `calc-zed` does not implement grammar behavior itself.
## Boundary
- `tree-sitter-calc` owns syntax recognition.
- `calc-core` owns calculation semantics.
- `calc-lsp` owns LSP behavior.
- `calc-zed` owns Zed extension registration and language server startup.
- `tree-sitter-calc` must not duplicate evaluation behavior from `calc-core`.
