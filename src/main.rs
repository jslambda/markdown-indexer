// file name: main.rs
use mdparser_exp::{JsonDocumentElement, index_markdown};
use serde_json;
use std::{env, fs, io, path::Path};

fn main() -> Result<(), markdown::message::Message> {
    // Usage: program <input_folder_or_markdown_file> [--depth N]
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        eprintln!(
            "Usage: {} <input_folder_or_markdown_file> [--depth N]",
            args[0]
        );
        std::process::exit(1);
    }

    let input = &args[1];
    let mut max_depth: Option<usize> = None;

    // Optional argument: --depth N or -d N
    if args.len() == 4 {
        let flag = &args[2];
        let value = &args[3];
        if flag == "--depth" || flag == "-d" {
            max_depth = Some(value.parse::<usize>().unwrap_or_else(|_| {
                eprintln!("Invalid depth value: {}", value);
                std::process::exit(1);
            }));
        } else {
            eprintln!("Unknown flag: {}", flag);
            std::process::exit(1);
        }
    }

    let input_path = Path::new(input);
    if !input_path.exists() {
        eprintln!("Input path does not exist: {}", input_path.display());
        std::process::exit(1);
    }

    let mut all_docs: Vec<JsonDocumentElement> = Vec::new();
    // pass starting depth = 0
    process_path(input_path, &mut all_docs, 0, max_depth)?;

    serde_json::to_writer_pretty(io::stdout(), &all_docs).expect("failed to serialize JSON");
    println!();
    Ok(())
}

/// `current_depth`: which level of recursion we are in (root = 0)
/// `max_depth`: Some(N) means N is maximum allowed depth, None means infinite
fn process_path(
    path: &Path,
    docs: &mut Vec<JsonDocumentElement>,
    current_depth: usize,
    max_depth: Option<usize>,
) -> Result<(), markdown::message::Message> {
    // If a max depth is defined and we are past it, stop recursion
    if let Some(limit) = max_depth {
        if current_depth > limit {
            return Ok(());
        }
    }

    if path.is_dir() {
        let entries = fs::read_dir(path).unwrap_or_else(|err| {
            eprintln!("Failed to read directory {}: {}", path.display(), err);
            std::process::exit(1);
        });

        for entry in entries {
            let entry = entry.unwrap_or_else(|err| {
                eprintln!(
                    "Failed to read directory entry in {}: {}",
                    path.display(),
                    err
                );
                std::process::exit(1);
            });

            let child_path = entry.path();
            process_path(&child_path, docs, current_depth + 1, max_depth)?;
        }
    } else if is_markdown_file(path) {
        let src = fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Failed to read {}: {}", path.display(), err);
            std::process::exit(1);
        });

        let sections = index_markdown(&src)?;
        let file_path = path.to_string_lossy().to_string();

        let file_docs: Vec<JsonDocumentElement> = sections
            .into_iter()
            .map(|s| JsonDocumentElement {
                file_path: file_path.clone(),
                header: s.title,
                text_blocks: s.body_text,
                code_blocks: s.code_blocks.into_iter().map(|cb| cb.value).collect(),
            })
            .collect();

        docs.extend(file_docs);
    }

    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") => true,
        _ => false,
    }
}
