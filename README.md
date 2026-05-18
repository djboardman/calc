# Calc

Calc is a small calculator language for editor-based calculations.

The project is split into:

- `calc-core`: calculator parsing, evaluation, and incremental document evaluation.
- `calc-lsp`: Language Server Protocol support for editor integrations.
- `calc-zed`: Zed extension wrapper that starts `calc-lsp`.
- `tree-sitter-calc`: Tree-sitter grammar for `.calc` files.

## Prerequisites

- Rust installed with `rustup`
- Node.js and npm, for generating the Tree-sitter parser
- Zed, for local extension testing

Install the Zed extension target:

```sh
rustup target add wasm32-wasip1
```

## Build

From the repository root:

```sh
cargo build
cargo build -p calc-lsp
cargo build -p calc-zed --target wasm32-wasip1
```

## Generate Tree-sitter Parser

The generated parser files live in `tree-sitter-calc/src`.

```sh
cd tree-sitter-calc
npx tree-sitter-cli generate
```

You can smoke-test the grammar with:

```sh
npx tree-sitter-cli parse /dev/stdin <<'EOF'
price = 10
tax = price * 0.2
price + tax
EOF
```

## Verify

From the repository root:

```sh
cargo fmt --all --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Install In Zed For Local Development

Build the language server and extension:

```sh
cargo build -p calc-lsp
cargo build -p calc-zed --target wasm32-wasip1
```

Then in Zed:

1. Open the Extensions view.
2. Choose `Install Dev Extension`.
3. Select this directory:

```text
/Users/david/src/tries/2026-05-17-calc/calc/crates/calc-zed
```

Open a `.calc` file from the workspace root. During local development, `calc-zed` looks for the language server in this order:

1. `crates/calc-zed/bin/calc-lsp`
2. `target/debug/calc-lsp`
3. `calc-lsp` on `PATH`

For normal local development, `cargo build -p calc-lsp` creates:

```text
target/debug/calc-lsp
```

## Example

Create a file named `example.calc`:

```calc
price = 10
tax = price * 0.2
price + tax
```

Zed should recognize it as Calc, start `calc-lsp`, show diagnostics for invalid calculations, and provide completion results for the current line.
