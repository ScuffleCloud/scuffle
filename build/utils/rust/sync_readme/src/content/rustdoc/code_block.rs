use std::borrow::Cow;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

pub(super) fn convert<'a, 'b>(events: impl IntoIterator<Item = Event<'a>> + 'b) -> impl Iterator<Item = Event<'a>> + 'b {
    let mut in_codeblock = None;
    events.into_iter().map(move |mut event| {
        if let Some(is_rust) = in_codeblock {
            match &mut event {
                Event::Text(text) => {
                    if !text.ends_with('\n') {
                        // workaround for https://github.com/Byron/pulldown-cmark-to-cmark/issues/48
                        *text = format!("{text}\n").into();
                    }
                    if is_rust {
                        // Hide lines starting with any number of whitespace
                        // followed by `# ` (comments), or just `#`. But `## `
                        // should be converted to `# `.
                        *text = text
                            .lines()
                            .filter_map(|line| {
                                // Adapted from
                                // https://github.com/rust-lang/rust/blob/942db6782f4a28c55b0b75b38fd4394d0483390f/src/librustdoc/html/markdown.rs#L169-L182.
                                let trimmed = line.trim();
                                if trimmed.starts_with("##") {
                                    // It would be nice to reuse
                                    // `pulldown_cmark::CowStr` here, but (at
                                    // least as of version 0.12.2) it doesn't
                                    // support collecting into a `String`.
                                    Some(Cow::Owned(line.replacen("##", "#", 1)))
                                } else if trimmed.starts_with("# ") {
                                    // Hidden line.
                                    None
                                } else if trimmed == "#" {
                                    // A plain # is a hidden line.
                                    None
                                } else {
                                    Some(Cow::Borrowed(line))
                                }
                            })
                            .flat_map(|line| [line, Cow::Borrowed("\n")])
                            .collect::<String>()
                            .into();
                    }
                }
                Event::End(TagEnd::CodeBlock) => {}
                _ => unreachable!(),
            }
        }

        match &mut event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let is_rust;
                match kind {
                    CodeBlockKind::Indented => {
                        is_rust = true;
                        *kind = CodeBlockKind::Fenced("rust".into());
                    }
                    CodeBlockKind::Fenced(tag) => {
                        is_rust = update_codeblock_tag(tag);
                    }
                }

                assert!(in_codeblock.is_none());
                in_codeblock = Some(is_rust);
            }
            Event::End(TagEnd::CodeBlock) => {
                assert!(in_codeblock.is_some());
                in_codeblock = None;
            }
            _ => {}
        }
        event
    })
}
fn is_attribute_tag(tag: &str) -> bool {
    // https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes
    // to support future rust edition, `edition\d{4}` treated as attribute tag
    matches!(tag, "" | "ignore" | "should_panic" | "no_run" | "compile_fail")
        || tag
            .strip_prefix("edition")
            .map(|x| x.len() == 4 && x.chars().all(|ch| ch.is_ascii_digit()))
            .unwrap_or_default()
}

fn update_codeblock_tag(tag: &mut CowStr<'_>) -> bool {
    let mut tag_count = 0;
    let is_rust = tag.split(',').filter(|tag| !is_attribute_tag(tag)).all(|tag| {
        tag_count += 1;
        tag == "rust"
    });
    if is_rust && tag_count == 0 {
        if tag.is_empty() {
            *tag = "rust".into();
        } else {
            *tag = format!("rust,{tag}").into();
        }
    }
    is_rust
}
