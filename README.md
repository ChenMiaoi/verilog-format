# Verilog Format

Rust CLI for formatting Verilog files.

## Usage

```text
Usage: verilog-format [OPTIONS]

Options:
  -f, --format <pathname>                 Verilog file to format
  -p, --print                             Print formatted output instead of overwriting the file
  -s, --settings <.verilog-format.yaml>  Explicit YAML settings file
  -v, --version                           Print version
  -h, --help                              Print help
```

## Configuration

The formatter now uses `.verilog-format.yaml`.

Resolution order:

1. If `--settings` is provided, use that file.
2. Otherwise, if the current working directory contains `.verilog-format.yaml`, use it automatically.
3. Otherwise, use built-in defaults.

Example:

```yaml
indent_width: 4
indent_type: space
spaces_before_trailing_comments: 1
spaces_after_trailing_comments: 0
spaces_before_if_statement: 1
spaces_blocking_assignment: 1
spaces_no_blocking_assignment: 1
spaces_in_parentheses: false
spaces_in_square_brackets: false
align_blocking_assignments: true
align_no_blocking_assignments: true
align_line_comments: false
```

Legacy property-style keys are still accepted in YAML for transition, for example `IndentWidth` and `AlignLineComments`.

## Examples

Print a formatted file:

```sh
verilog-format -p -f input_file.v
```

Format a file in place with an explicit config:

```sh
verilog-format -f input_file.v -s path/to/.verilog-format.yaml
```

Format a file using the config from the current working directory:

```sh
verilog-format -f input_file.v
```

## Build

```sh
cargo build --release
```

The binary will be generated at:

```text
target/release/verilog-format
target/release/verilog-format.exe
```

## Publish Dry Run

Validate that the crate can be published without actually uploading it:

```sh
cargo publish --dry-run
```

## CI

GitHub Actions now runs a Rust-native workflow on both Linux and Windows:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --locked`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked` on Linux and Windows

Tag pushes matching `v*` publish Linux tarballs, a directly downloadable Linux binary, Windows zip packages, and a directly downloadable Windows `.exe`.
