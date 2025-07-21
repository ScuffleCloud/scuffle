use std::{borrow::Cow, iter, ops::Range};

use super::{super::contents::Contents, Marker, Replace};

pub(in super::super) fn replace_all(
    text: &str,
    markers: &[(Replace, Range<usize>)],
    contents: &[Contents],
) -> String {
    let pairs = markers
        .iter()
        .zip(contents)
        .map(|((replace, range), contents)| ((replace.clone(), contents), range.clone()));

    interpolate_ranges(0..text.len(), pairs)
        .map(|(contents, range)| match contents {
            Some((replace, contents)) => {
                if contents.text().is_empty() {
                    Cow::Owned(format!("{}\n", Marker::Replace(replace)))
                } else {
                    Cow::Owned(format!(
                        "{}\n{}{}\n",
                        Marker::Start(replace),
                        contents.text(),
                        Marker::End
                    ))
                }
            }
            None => Cow::Borrowed(&text[range]),
        })
        .collect()
}

fn interpolate_ranges<T>(
    range: Range<usize>,
    items: impl IntoIterator<Item = (T, Range<usize>)>,
) -> impl Iterator<Item = (Option<T>, Range<usize>)> {
    let mut items = items.into_iter().peekable();
    let mut offset = range.start;
    iter::from_fn(move || match items.peek() {
        Some(&(_, Range { start, .. })) if offset < start => {
            let range = offset..start;
            offset = start;
            Some((None, range))
        }
        Some(_) => {
            let (item, Range { start, end }) = items.next().unwrap();
            offset = end;
            Some((Some(item), start..end))
        }
        None if offset < range.end => {
            let range = offset..range.end;
            offset = range.end;
            Some((None, range))
        }
        None => None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn interpolate_ranges() {
        let items = [(1, 0..1), (2, 1..2), (3, 2..3)];
        let ranges = super::interpolate_ranges(0..3, items);
        assert_eq!(
            ranges.collect::<Vec<_>>(),
            vec![(Some(1), 0..1), (Some(2), 1..2), (Some(3), 2..3),]
        );

        let items = [(1, 3..4), (2, 4..5), (3, 6..7), (4, 8..9)];
        let ranges = super::interpolate_ranges(0..10, items);
        assert_eq!(
            ranges.collect::<Vec<_>>(),
            vec![
                (None, 0..3),
                (Some(1), 3..4),
                (Some(2), 4..5),
                (None, 5..6),
                (Some(3), 6..7),
                (None, 7..8),
                (Some(4), 8..9),
                (None, 9..10),
            ]
        );
    }
}
