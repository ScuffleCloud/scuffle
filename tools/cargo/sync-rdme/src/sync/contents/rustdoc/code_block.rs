use std::borrow::Cow;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

pub(super) fn convert<'a, 'b>(
    events: impl IntoIterator<Item = Event<'a>> + 'b,
) -> impl Iterator<Item = Event<'a>> + 'b {
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
    matches!(
        tag,
        "" | "ignore" | "should_panic" | "no_run" | "compile_fail"
    ) || tag
        .strip_prefix("edition")
        .map(|x| x.len() == 4 && x.chars().all(|ch| ch.is_ascii_digit()))
        .unwrap_or_default()
}

fn update_codeblock_tag(tag: &mut CowStr<'_>) -> bool {
    let mut tag_count = 0;
    let is_rust = tag
        .split(',')
        .filter(|tag| !is_attribute_tag(tag))
        .all(|tag| {
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

#[cfg(test)]
mod tests {
    #[test]
    fn update_codeblock_tag() {
        fn check(tag: &str, expected_tag: &str, expected_is_rust: bool) {
            let mut tag = tag.into();
            let is_rust = super::update_codeblock_tag(&mut tag);
            assert_eq!(tag.as_ref(), expected_tag);
            assert_eq!(is_rust, expected_is_rust)
        }
        check("", "rust", true);
        check("typescript", "typescript", false);
        check("rust", "rust", true);
        check("ignore", "rust,ignore", true);
        check("ignore,rust", "ignore,rust", true);
        check("ignore,typescript", "ignore,typescript", false);

        check(
            "ignore,should_panic,no_run,compile_fail,edition2015,edition2018,edition2021",
            "rust,ignore,should_panic,no_run,compile_fail,edition2015,edition2018,edition2021",
            true,
        );
        check("edition9999", "rust,edition9999", true);
        check("edition99999", "edition99999", false);
        check("editionabcd", "editionabcd", false);
    }

    #[test]
    fn hide_codeblock_line() {
        let input = r#"
Lorem ipsum

```rust
# This line and the next should be hidden, but the following should not.
#
#[derive(Debug)]
struct Foo;

fn main() {
    # As should this and the next line.
    #
    #But not this.
    ## This should become a single #.
    ##And this.
}
```

```toml
# This is not Rust so it should not be hidden.
```
"#;

        let expected = r#"Lorem ipsum

````rust
#[derive(Debug)]
struct Foo;

fn main() {
    #But not this.
    # This should become a single #.
    #And this.
}
````

````toml
# This is not Rust so it should not be hidden.
````"#;

        let events: Vec<_> = pulldown_cmark::Parser::new(input).collect();
        let events: Vec<_> = super::convert(events).collect();

        let mut output = String::new();
        pulldown_cmark_to_cmark::cmark(events.into_iter(), &mut output).unwrap();
        assert_eq!(output, expected, "output matches expected");
    }
}
