# markdown-indexer

`markdown-indexer` is a small CLI that walks a markdown file or directory of markdown documents and emits a structured JSON index. It is useful for creating searchable corpora of notes, documentation, or blog posts without writing a custom crawler.

## Installation

This project is built with Rust. To build and run it you need a recent [Rust toolchain](https://www.rust-lang.org/tools/install).

```bash
cargo build --release
```

The compiled binary will be available at `target/release/mdparser-exp`.

## Usage

From the repository root run the CLI with a markdown file or directory as the first argument:

```bash
cargo run -- <input_path> [--depth N]
```

- `<input_path>` can be a single `.md`/`.markdown` file or a directory containing markdown files.
- `--depth N` (or `-d N`) is optional and limits how deep the directory traversal should recurse. When omitted, traversal is unbounded.

If the input path does not exist or an unknown flag is provided, the program prints an error message and exits with a non-zero status.

### Output format

The command prints a JSON array to `stdout`. Each element represents a header section from one of the parsed markdown files:

- `file_path`: Absolute or relative path to the source markdown file.
- `header`: The section title.
- `text_blocks`: An array of text paragraphs under the section.
- `code_blocks`: An array of code block contents extracted from the section.

### Examples

Index a single file:

```bash
cargo run -- README.md
```

Index a directory but only descend two levels deep:

```bash
cargo run -- docs --depth 2
```

Redirect the JSON output to a file for later processing:

```bash
cargo run -- notes > index.json
```

## Development

The main CLI entrypoint lives in [`src/main.rs`](src/main.rs). The parser utilities are provided by the `mdparser_exp` crate dependencies declared in [`Cargo.toml`](Cargo.toml). There are no additional runtime requirements.
