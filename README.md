# Calc

Calc is a small calculator language for editor-based calculations.

The project is split into:

- `calc-core`: calculator parsing, evaluation, and incremental document evaluation.
- `calc-lsp`: Language Server Protocol support for editor integrations.
- `calc-zed`: Zed extension wrapper that starts `calc-lsp`.
- `tree-sitter-calc`: external Tree-sitter grammar for `.calc` files, referenced by `calc-zed`.

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

## Tree-sitter Grammar

The Tree-sitter grammar is fetched by Zed from the repository and revision pinned in:

```text
crates/calc-zed/extension.toml
```

The grammar is not built by the main Calc workspace. If you are developing the grammar repository separately, generate and smoke-test it from that repository:

```sh
cd ../tree-sitter-calc
npx tree-sitter-cli generate
npx tree-sitter-cli parse /dev/stdin <<'EOF'
price = 10
tax = price * 0.2
price + tax
EOF
```

If the smoke test warns that no parser directories are configured, initialize the local Tree-sitter CLI config:

```sh
npx tree-sitter-cli init-config
```

That warning is local CLI setup only; it is not required for generating the parser or building the project.

## Verify

From the repository root:

```sh
cargo fmt --all --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Install In Zed For Local Testing

From the Calc repository root, install the language server on `PATH` and build the Zed extension:

```sh
cargo install --path crates/calc-lsp
cargo build -p calc-zed --target wasm32-wasip1
```

`calc-zed` starts `calc-lsp` by looking in this order:

1. `crates/calc-zed/bin/calc-lsp`
2. `calc-lsp` on `PATH`

For local testing, `cargo install --path crates/calc-lsp` installs:

```text
~/.cargo/bin/calc-lsp
```

Then in Zed:

1. Open the Extensions view.
2. Choose `Install Dev Extension`.
3. Select this directory:

```text
/Users/david/src/tries/2026-05-17-calc/calc/crates/calc-zed
```

Open a `.calc` file from any workspace. Zed should recognize it as Calc, start `calc-lsp`, show diagnostics for invalid calculations, provide completions, and show inlay hints when Zed inlay hints are enabled.

For Calc totals, enable Zed's other inlay hints:

```json
{
  "inlay_hints": {
    "enabled": true,
    "show_other_hints": true
  }
}
```

## Example

Create a file named `example.calc`:

```calc
price = 10
tax = price * 0.2
price + tax
```

Zed should show diagnostics, completions for previously declared variables, and inlay totals when inlay hints are enabled.

## Updating An Installed Dev Extension

After changing `calc-core`, `calc-lsp`, or `calc-zed`, rebuild the affected local pieces:

```sh
cargo install --path crates/calc-lsp
cargo build -p calc-zed --target wasm32-wasip1
```

Then use Zed's extension `Rebuild` button for the installed Calc dev extension, or reinstall the dev extension from:

```text
/Users/david/src/tries/2026-05-17-calc/calc/crates/calc-zed
```

If Zed still uses an old language server process, restart Zed.

After changing the external Tree-sitter grammar, regenerate the parser, commit the grammar repository, and get the new commit:

```sh
cd ../tree-sitter-calc
npx tree-sitter-cli generate
git add grammar.js tree-sitter.json src
git commit -m "Update calc grammar"
git rev-parse HEAD
```

Then update `crates/calc-zed/extension.toml` so `[grammars.calc].rev` matches the new grammar commit.

If Zed has already installed the dev extension, remove any stale local grammar checkout before reinstalling:

```sh
rm -rf crates/calc-zed/grammars
```

Then rebuild or reinstall the dev extension in Zed.
