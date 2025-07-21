use std::{ops::Range, sync::Arc};

use miette::{NamedSource, SourceSpan};
use pulldown_cmark::Event;

use crate::sync::ManifestFile;

use super::{super::MarkdownFile, Marker, ParseMarkerError, Replace};

pub(in super::super) fn find_all<'events>(
    readme: &MarkdownFile,
    manifest: &ManifestFile,
    events: impl IntoIterator<Item = (Event<'events>, Range<usize>)> + 'events,
) -> Result<Vec<(Replace, Range<usize>)>, FindAllError> {
    let events = events.into_iter();
    let it = Iter { manifest, events };
    let mut markers = vec![];
    let mut errors = vec![];
    for res in it {
        match res {
            Ok(marker) => markers.push(marker),
            Err(err) => errors.push(err),
        }
    }

    if !errors.is_empty() {
        let source_code = readme.to_named_source();
        return Err(FindAllError {
            source_code,
            errors,
        });
    }

    Ok(markers)
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("failed to parse README")]
pub(in super::super) struct FindAllError {
    #[source_code]
    source_code: NamedSource<Arc<str>>,
    #[related]
    errors: Vec<FindError>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
enum FindError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseMarker(#[from] ParseMarkerError),
    #[error("unexpected end marker")]
    UnexpectedEndMarker {
        #[label = "the end marker defined here"]
        span: SourceSpan,
    },
    #[error("corresponding end marker not found")]
    EndMarkerNotFound {
        #[label = "the start label defined here"]
        start_span: SourceSpan,
    },
    #[error("nested markers are not allowed")]
    NestedMarker {
        #[label = "the nested marker defined here"]
        nested_span: SourceSpan,
        #[label = "the previous marker starts here"]
        previous_span: SourceSpan,
    },
}

#[derive(Debug)]
struct Iter<'manifest, I> {
    manifest: &'manifest ManifestFile,
    events: I,
}

macro_rules! itry {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                return Some(Err(e));
            }
        }
    };
}

impl<'event, I> Iterator for Iter<'_, I>
where
    I: Iterator<Item = (Event<'event>, Range<usize>)>,
{
    type Item = Result<(Replace, Range<usize>), FindError>;

    fn next(&mut self) -> Option<Self::Item> {
        match itry!(self.next_marker())? {
            (Marker::Replace(replace), range) => Some(Ok((replace, range))),
            (Marker::Start(replace), start_range) => match itry!(self.next_marker()) {
                Some((Marker::End, end_range)) => {
                    Some(Ok((replace, start_range.start..end_range.end)))
                }
                Some((_, nested_range)) => Some(Err(FindError::NestedMarker {
                    nested_span: nested_range.into(),
                    previous_span: start_range.into(),
                })),
                None => Some(Err(FindError::EndMarkerNotFound {
                    start_span: start_range.into(),
                })),
            },
            (Marker::End, range) => {
                Some(Err(FindError::UnexpectedEndMarker { span: range.into() }))
            }
        }
    }
}

impl<'event, I> Iter<'_, I>
where
    I: Iterator<Item = (Event<'event>, Range<usize>)>,
{
    fn next_marker(&mut self) -> Result<Option<(Marker, Range<usize>)>, FindError> {
        for (event, range) in self.events.by_ref() {
            if let Event::Html(html) = &event {
                if let Some(marker) = Marker::matches((html, range.clone().into()), self.manifest)?
                {
                    return Ok(Some((marker, range)));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::Parser;

    use super::*;

    fn line_ranges(lines: &[impl AsRef<str>]) -> Vec<Range<usize>> {
        lines
            .iter()
            .scan(0, |offset, line| {
                let line = line.as_ref();
                let range = *offset..*offset + line.len() + 1;
                *offset = range.end;
                Some(range)
            })
            .collect()
    }

    #[test]
    fn no_markers() {
        let input = "Hello, world!";
        let mut markers = Iter {
            manifest: &ManifestFile::dummy(Default::default()),
            events: Parser::new(input).into_offset_iter(),
        };
        assert!(markers.next().is_none());
    }

    #[test]
    fn replace_marker() {
        let lines = [
            "Good morning, world!".to_string(),
            Marker::Replace(Replace::Title).to_string(),
            "Good afternoon, world!".to_string(),
            Marker::Replace(Replace::Badge {
                name: "".into(),
                badges: vec![].into(),
            })
            .to_string(),
            "Good evening, world!".to_string(),
            Marker::Replace(Replace::Rustdoc).to_string(),
            "Good night, world!".to_string(),
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc::indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let mut markers = Iter {
            manifest: &ManifestFile::dummy(toml::from_str(config).unwrap()),
            events: Parser::new(&input).into_offset_iter(),
        };
        assert_eq!(
            markers.next().unwrap().unwrap(),
            (Replace::Title, ranges[1].clone())
        );
        assert_eq!(
            markers.next().unwrap().unwrap(),
            (
                Replace::Badge {
                    name: "".into(),
                    badges: vec![].into()
                },
                ranges[3].clone()
            )
        );
        assert_eq!(
            markers.next().unwrap().unwrap(),
            (Replace::Rustdoc, ranges[5].clone())
        );
        assert!(markers.next().is_none());
    }

    #[test]
    fn replace_region() {
        let lines = [
            "Good morning, world!".to_string(),
            Marker::Start(Replace::Title).to_string(),
            "Good afternoon, world!".to_string(),
            "# Heading!".to_string(),
            Marker::End.to_string(),
            "Good evening, world!".to_string(),
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc::indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let mut markers = Iter {
            manifest: &ManifestFile::dummy(toml::from_str(config).unwrap()),
            events: Parser::new(&input).into_offset_iter(),
        };
        assert_eq!(
            markers.next().unwrap().unwrap(),
            (Replace::Title, ranges[1].start..ranges[4].end)
        );
        assert!(markers.next().is_none());
    }
}
