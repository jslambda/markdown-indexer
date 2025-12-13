use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub lang: Option<String>,
    pub meta: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub level: u8,
    pub body_text: Vec<String>,
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JsonDocumentElement {
    pub file_path: String,
    pub header: String,
    pub text_blocks: Vec<String>,
    pub code_blocks: Vec<String>,
}

use markdown::message::Message;
use markdown::{self, ParseOptions, mdast};

/// Parse a markdown document into sections, each starting at a heading.
/// All text / code until the next heading belongs to that section.
///
/// Sections are *flat*: nested headings become separate sections, but
/// each still carries its heading level (`#` = 1, `##` = 2, …).

pub fn index_markdown(src: &str) -> Result<Vec<Section>, Message> {
    let ast = markdown::to_mdast(src, &ParseOptions::default())?;

    let root = match ast {
        mdast::Node::Root(root) => root,
        _ => unreachable!("to_mdast() always returns a Root at the top"),
    };

    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;

    for node in &root.children {
        match node {
            // === Headings start a new section ===
            mdast::Node::Heading(h) => {
                // Finish previous section, if any.
                if let Some(sec) = current.take() {
                    sections.push(sec);
                }

                let title = node_to_plain_text(node);

                current = Some(Section {
                    title,
                    level: h.depth,
                    body_text: Vec::new(),
                    code_blocks: Vec::new(),
                });
            }

            // === Paragraphs become body text ===
            mdast::Node::Paragraph(_) => {
                let text = node_to_plain_text(node);
                if text.trim().is_empty() {
                    continue;
                }

                if let Some(sec) = current.as_mut() {
                    if !sec.body_text.is_empty() {
                        //sec.body_text.push_str("\n\n");
                    }
                    sec.body_text.push(text);
                } else {
                    // Content before the first heading -> preamble section
                    let preamble = Section {
                        title: String::from("(preamble)"),
                        level: 0,
                        body_text: vec![text],
                        code_blocks: Vec::new(),
                    };
                    current = Some(preamble);
                }
            }

            // === Top-level fenced code blocks ===
            mdast::Node::Code(code) => {
                if let Some(sec) = current.as_mut() {
                    sec.code_blocks.push(CodeBlock {
                        lang: code.lang.clone(),
                        meta: code.meta.clone(),
                        value: code.value.clone(),
                    });
                } else {
                    // Code before any heading -> attach to a synthetic preamble section
                    let sec = Section {
                        title: String::from("(preamble)"),
                        level: 0,
                        body_text: Vec::new(),
                        code_blocks: vec![CodeBlock {
                            lang: code.lang.clone(),
                            meta: code.meta.clone(),
                            value: code.value.clone(),
                        }],
                    };
                    current = Some(sec);
                }
            }

            // === Block/inline content we treat as extra text ===
            //
            // We just flatten them to plain text and append to current section /
            // preamble. `node_to_plain_text` will walk their children.
            mdast::Node::Blockquote(_)
            | mdast::Node::FootnoteDefinition(_)
            | mdast::Node::MdxJsxFlowElement(_)
            | mdast::Node::List(_)
            | mdast::Node::MdxjsEsm(_)
            | mdast::Node::Toml(_)
            | mdast::Node::Yaml(_)
            | mdast::Node::Math(_)
            | mdast::Node::MdxFlowExpression(_)
            | mdast::Node::Table(_)
            | mdast::Node::TableRow(_)
            | mdast::Node::TableCell(_)
            | mdast::Node::ListItem(_)
            | mdast::Node::Definition(_)
            | mdast::Node::ThematicBreak(_)
            | mdast::Node::Html(_)
            // Phrasing / inline-like nodes (normally don’t show up at root,
            // but we handle them anyway to make the match exhaustive):
            | mdast::Node::Break(_)
            | mdast::Node::InlineCode(_)
            | mdast::Node::InlineMath(_)
            | mdast::Node::Delete(_)
            | mdast::Node::Emphasis(_)
            | mdast::Node::MdxTextExpression(_)
            | mdast::Node::FootnoteReference(_)
            | mdast::Node::Image(_)
            | mdast::Node::ImageReference(_)
            | mdast::Node::MdxJsxTextElement(_)
            | mdast::Node::Link(_)
            | mdast::Node::LinkReference(_)
            | mdast::Node::Strong(_)
            | mdast::Node::Text(_) => {
                let text = node_to_plain_text(node);
                if text.trim().is_empty() {
                    continue;
                }

                if let Some(sec) = current.as_mut() {
                    if !sec.body_text.is_empty() {
                        //sec.body_text.push_str("\n\n");
                    }
                    sec.body_text.push(text);
                } else {
                    let preamble = Section {
                        title: String::from("(preamble)"),
                        level: 0,
                        body_text: vec![text],
                        code_blocks: Vec::new(),
                    };
                    current = Some(preamble);
                }
            }

            // Root should not appear as a child of Root, but we include it
            // so the match is truly exhaustive and future-proof.
            mdast::Node::Root(_) => {
                // no-op
            }
        }
    }

    // Flush last section.
    if let Some(sec) = current.take() {
        sections.push(sec);
    }

    Ok(sections)
}
/// Collect human-readable text from a node (drops formatting, links, etc.).
fn node_to_plain_text(node: &mdast::Node) -> String {
    let mut out = String::new();
    collect_text(node, &mut out);
    out
}

fn collect_text(node: &mdast::Node, out: &mut String) {
    match node {
        mdast::Node::Text(t) => {
            out.push_str(&t.value);
        }
        mdast::Node::InlineCode(c) => {
            // Treat inline code as text for indexing
            out.push_str(&c.value);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    collect_text(child, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_paragraph_and_code_are_grouped_into_one_section() {
        let src = r#"
# Introduction


This is the intro text.


```rust
fn main() {
println!(\"hello\");
}
```
"#;

        let sections = index_markdown(src).expect("parse ok");

        assert_eq!(sections.len(), 1);
        println!("{:?}", sections);

        let s = &sections[0];
        assert_eq!(s.title, "Introduction");
        assert_eq!(s.level, 1);

        // Body text
        assert!(
            s.body_text.contains(&"This is the intro text.".to_string()),
            "body_text = {:?}",
            s.body_text
        );

        // Code block
        assert_eq!(s.code_blocks.len(), 1);
        let cb = &s.code_blocks[0];
        assert_eq!(cb.lang.as_deref(), Some("rust"));
        assert!(cb.value.contains("println!("));
    }

    #[test]
    fn multiple_headings_become_multiple_sections() {
        let src = r#"
# Intro


Intro text.


## Details


More details here.


```python
print(\"hi\")
```
"#;

        let sections = index_markdown(src).expect("parse ok");

        assert_eq!(sections.len(), 2);

        let intro = &sections[0];
        assert_eq!(intro.title, "Intro");
        assert_eq!(intro.level, 1);
        assert!(intro.body_text.contains(&"Intro text.".to_string()));

        let details = &sections[1];
        assert_eq!(details.title, "Details");
        assert_eq!(details.level, 2);
        assert!(details.body_text[0].contains(&"More details here".to_string()));
        assert_eq!(details.code_blocks.len(), 1);
        assert_eq!(details.code_blocks[0].lang.as_deref(), Some("python"));
        // assert!(details.code_blocks[0].value.contains("print(\"hi\")"));
    }
    #[test]
    fn content_before_first_heading_goes_into_preamble_section() {
        let src = r#"
This is some text before any heading.


```bash
echo \"hello\"
```


# Heading


More text under heading.
"#;

        let sections = index_markdown(src).expect("parse ok");

        // We expect: (preamble), then Heading
        assert_eq!(sections.len(), 2);

        let preamble = &sections[0];
        assert_eq!(preamble.title, "(preamble)");
        assert_eq!(preamble.level, 0);
        assert!(preamble.body_text[0].contains(&"text before any heading".to_string()));
        assert_eq!(preamble.code_blocks.len(), 1);
        assert_eq!(preamble.code_blocks[0].lang.as_deref(), Some("bash"));
        // assert!(preamble.code_blocks[0].value.contains("echo \"hello\""));

        let heading = &sections[1];
        assert_eq!(heading.title, "Heading");
        assert_eq!(heading.level, 1);
        assert!(heading.body_text[0].contains(&"More text under heading".to_string()));
    }

    #[test]
    fn inline_formatting_is_flattened_to_plain_text() {
        let src = r#"
# Title


This is *bold* and `inline_code` and a [link](https://example.com).
    "#;

        let sections = index_markdown(src).expect("parse ok");

        assert_eq!(sections.len(), 1);
        let s = &sections[0];

        assert_eq!(s.title, "Title");
        let body = &s.body_text;

        // Our node_to_plain_text should keep the words, drop formatting
        assert!(body[0].contains(&"This is".to_string()), "{:?}", body);
        assert!(body[0].contains(&"bold".to_string()), "{:?}", body);
        assert!(body[0].contains(&"inline_code".to_string()));
        assert!(body[0].contains(&"link".to_string()));
    }
}
