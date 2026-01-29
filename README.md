# markdown-indexer

`markdown-indexer` is a small CLI that walks a markdown file or directory of markdown documents and emits a structured JSON index. It is useful for creating searchable corpora of notes, documentation, or blog posts without writing a custom crawler.

## Installation

This project is built with Rust. To build and run it you need a recent [Rust toolchain](https://www.rust-lang.org/tools/install).

```bash
cargo build --release
```

The compiled binary will be available at `target/release/mdparser-exp`.

## Usage

From the repository root run the CLI with one or more markdown files or directories as positional arguments:

```bash
cargo run -- <input1> [input2 ...] [--depth N]
```

- Each input can be a `.md`/`.markdown` file or a directory containing markdown files.
- The optional `--depth N`/`-d N` flag limits how deep directory traversal should recurse. When omitted, traversal is unbounded.
- Flags must appear **after** all inputs; a leading `--depth` or unknown flag results in an error.
- Each path is validated before processing. Missing paths are listed and cause the command to exit with a non-zero status.

If any input path does not exist or an unknown flag is provided, the program prints an error message and exits with a non-zero status.

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

Index multiple inputs and emit a single combined JSON document:

```bash
cargo run -- docs notes/guide.md blog_posts
```

Index a directory but only descend two levels deep (note the depth flag trails the inputs):

```bash
cargo run -- docs --depth 2
```

Redirect the JSON output to a file for later processing:

```bash
cargo run -- notes > index.json
```

## Library usage

The parser is also exposed as a library if you want to integrate it into your own Rust application. Add the crate to your `Cargo.toml` (use a path dependency when working from a local checkout, or swap in a published version if you depend on crates.io):

```toml
[dependencies]
mdparser-exp = { git = "https://github.com/jslambda/markdown-indexer" }
```

Then call `index_markdown` to receive the parsed sections and adapt them to your needs:

```rust
use mdparser_exp::index_markdown;

fn main() -> Result<(), markdown::message::Message> {
    let src = "# Title\n\nSome text with `inline` code.";
    let sections = index_markdown(src)?;

    for section in sections {
        println!("{} (level {})", section.title, section.level);
        println!("Paragraphs: {}", section.body_text.len());
        println!(
            "Code blocks: {}",
            section.code_blocks.iter().map(|cb| &cb.value).count()
        );
    }

    Ok(())
}
```

If you want the same JSON shape as the CLI, build your own `JsonDocumentElement` values from the returned sections (see `src/main.rs` for the conversion logic).

## Development

The main CLI entrypoint lives in [`src/main.rs`](src/main.rs). The parser utilities are provided by the `mdparser_exp` crate dependencies declared in [`Cargo.toml`](Cargo.toml). There are no additional runtime requirements.
