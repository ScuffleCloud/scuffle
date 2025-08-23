use std::str::FromStr;

use anyhow::Context;

use crate::content::Content;

static MARKER_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r#"<!-- sync-readme (\w+)?\s*(\[\[|\]\])? -->"#).expect("bad regex"));

#[derive(Debug)]
enum MarkerCategory {
    Title,
    Badge,
    Rustdoc,
}

impl std::fmt::Display for MarkerCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Title => f.write_str("title"),
            Self::Badge => f.write_str("badge"),
            Self::Rustdoc => f.write_str("rustdoc"),
        }
    }
}

impl FromStr for MarkerCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "title" => Ok(Self::Title),
            "badge" => Ok(Self::Badge),
            "rustdoc" => Ok(Self::Rustdoc),
            s => Err(anyhow::anyhow!("unknown marker {s}")),
        }
    }
}

#[derive(Debug)]
struct Marker {
    category: MarkerCategory,
    start: usize,
    end: usize,
}

pub fn render(readme: &str, content: &Content) -> anyhow::Result<String> {
    let mut markers = Vec::new();

    let mut iter = MARKER_REGEX.captures_iter(readme);

    while let Some(capture) = iter.next() {
        let marker = capture.get(0).expect("always a zero group");
        let category: MarkerCategory = capture
            .get(1)
            .map(|s| s.as_str().parse())
            .transpose()?
            .context("missing category")?;
        let open = capture.get(2).map(|c| c.as_str());

        let start = marker.start();
        let mut end = marker.end();
        if open == Some("[[") {
            let end_capture = iter.next().context("marker opens but never closes")?;
            let marker = end_capture.get(0).expect("always a zero group");
            anyhow::ensure!(
                end_capture.get(1).is_none() && end_capture.get(2).is_some_and(|c| c.as_str() == "]]"),
                "capture after an open must be a close: {}",
                marker.as_str(),
            );

            end = marker.end();
        }

        markers.push(Marker { category, start, end });
    }

    let mut readme_builder = String::new();
    let mut idx = 0;

    for marker in markers {
        readme_builder.push_str(&readme[idx..marker.start]);
        idx = marker.end + 1;

        use std::fmt::Write;

        let content = match marker.category {
            MarkerCategory::Badge => content.badge.as_str(),
            MarkerCategory::Rustdoc => content.rustdoc.as_str(),
            MarkerCategory::Title => content.title.as_str(),
        }
        .trim();

        if content.is_empty() {
            writeln!(&mut readme_builder, "<!-- sync-readme {} -->", marker.category).expect("write failed");
        } else {
            writeln!(
                &mut readme_builder,
                "<!-- sync-readme {} [[ -->\n{content}\n<!-- sync-readme ]] -->",
                marker.category
            )
            .expect("write failed");
        }
    }

    readme_builder.push_str(&readme[idx..]);

    Ok(readme_builder)
}
