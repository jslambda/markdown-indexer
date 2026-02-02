// file name: main.rs
use markdown2json::{JsonDocumentElement, index_markdown};
use serde_json;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

fn main() -> Result<(), markdown::message::Message> {
    let args: Vec<String> = env::args().collect();
    let (inputs, max_depth) = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let mut existing_inputs: Vec<PathBuf> = Vec::new();
    let mut missing_inputs: Vec<String> = Vec::new();

    for input in &inputs {
        let path = PathBuf::from(input);
        if path.exists() {
            existing_inputs.push(path);
        } else {
            missing_inputs.push(path.to_string_lossy().to_string());
        }
    }

    if !missing_inputs.is_empty() {
        eprintln!("The following input paths do not exist:");
        for missing in missing_inputs {
            eprintln!("  - {}", missing);
        }
        std::process::exit(1);
    }

    let mut all_docs: Vec<JsonDocumentElement> = Vec::new();

    for path in &existing_inputs {
        // pass starting depth = 0
        process_path(path, &mut all_docs, 0, max_depth)?;
    }

    serde_json::to_writer_pretty(io::stdout(), &all_docs).expect("failed to serialize JSON");
    println!();
    Ok(())
}

/// Parse CLI arguments into a list of input paths and an optional depth limit.
///
/// Expectations and validation rules:
/// - At least one positional input is required; the program name is at `args[0]`.
/// - The optional `--depth`/`-d` flag (with its numeric value) must appear **after**
///   all positional inputs. Any flag-like token before the final position triggers
///   an error so we can clearly tell users about ordering requirements.
/// - If the depth flag is provided without a following value, the function returns
///   a helpful error message rather than panicking.
fn parse_args(args: &[String]) -> Result<(Vec<String>, Option<usize>), String> {
    if args.len() < 2 {
        return Err(usage(&args[0]));
    }

    let mut depth: Option<usize> = None;
    let mut inputs_slice: &[String] = &args[1..];

    if let Some(last) = inputs_slice.last() {
        if last == "--depth" || last == "-d" {
            return Err("Expected a value after --depth/-d".to_string());
        }
    }

    if inputs_slice.len() >= 2 {
        let flag_candidate = &inputs_slice[inputs_slice.len() - 2];
        let value_candidate = &inputs_slice[inputs_slice.len() - 1];

        if flag_candidate == "--depth" || flag_candidate == "-d" {
            depth = Some(
                value_candidate
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid depth value: {}", value_candidate))?,
            );
            inputs_slice = &inputs_slice[..inputs_slice.len() - 2];
        }
    }

    if inputs_slice.is_empty() {
        return Err(usage(&args[0]));
    }

    for arg in inputs_slice {
        if arg.starts_with('-') {
            return Err(format!(
                "Unknown flag or flag placed before inputs: {}\n{}",
                arg,
                usage(&args[0])
            ));
        }
    }

    Ok((inputs_slice.to_vec(), depth))
}

fn usage(program: &str) -> String {
    format!(
        "Usage: {program} <input1> [input2 ...] [--depth N]\n  • Each input can be a markdown file or a folder.\n  • The optional --depth/-d flag must come after all inputs."
    )
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

#[cfg(test)]
mod tests {
    use super::parse_args;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_multiple_inputs_with_depth() {
        let argv = args(&["program", "input1", "input2", "--depth", "3"]);
        let (inputs, depth) = parse_args(&argv).expect("should parse");

        assert_eq!(inputs, vec!["input1", "input2"]);
        assert_eq!(depth, Some(3));
    }

    #[test]
    fn errors_on_missing_depth_value() {
        let argv = args(&["program", "input1", "--depth"]);
        let err = parse_args(&argv).expect_err("should error");

        assert!(err.contains("Expected a value after --depth/-d"));
    }

    #[test]
    fn errors_when_flag_precedes_inputs() {
        let argv = args(&["program", "-d", "2", "input1"]);
        let err = parse_args(&argv).expect_err("should error");

        assert!(err.contains("Unknown flag or flag placed before inputs"));
    }
}
