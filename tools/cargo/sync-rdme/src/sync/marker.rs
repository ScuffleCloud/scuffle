use std::{fmt, sync::Arc};

use miette::SourceSpan;

use crate::{metadata::BadgeItem, traits::StrSpanExt};

pub(super) use self::{find::*, replace::*};

use super::ManifestFile;

mod find;
mod replace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Replace {
    Title,
    Badge {
        name: Arc<str>,
        badges: Arc<[BadgeItem]>,
    },
    Rustdoc,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ParseReplaceError {
    #[error("unknown replace specifier: {0:?}")]
    UnknownReplace(String),
    #[error("badge group not configured in package manifest: package.metadata.cargo-sync-rdme.badge.badges{hyphen}{0}", hyphen = if .0.is_empty() { "" } else { "-" })]
    NoSuchBadgeGroup(String),
}

impl Replace {
    fn from_str(s: &str, manifest: &ManifestFile) -> Result<Self, ParseReplaceError> {
        let group = match s {
            "title" => return Ok(Self::Title),
            "rustdoc" => return Ok(Self::Rustdoc),
            "badge" => "",
            _ => {
                if let Some(group) = s.strip_prefix("badge:") {
                    group
                } else {
                    return Err(ParseReplaceError::UnknownReplace(s.to_owned()));
                }
            }
        };
        let badges = &manifest.value().config().badge.badges;
        let (name, badges) = badges
            .get_key_value(group)
            .ok_or_else(|| ParseReplaceError::NoSuchBadgeGroup(group.to_owned()))?;
        Ok(Self::Badge {
            name: Arc::clone(name),
            badges: Arc::clone(badges),
        })
    }
}

impl fmt::Display for Replace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Title => write!(f, "title"),
            Self::Badge { name, .. } => {
                if name.is_empty() {
                    write!(f, "badge")
                } else {
                    write!(f, "badge:{name}")
                }
            }
            Self::Rustdoc => write!(f, "rustdoc"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Marker {
    Replace(Replace),
    Start(Replace),
    End,
}

const MAGIC: &str = "cargo-sync-rdme";

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Replace(replace) => write!(f, "<!-- {MAGIC} {replace} -->"),
            Self::Start(replace) => write!(f, "<!-- {MAGIC} {replace} [[ -->"),
            Self::End => write!(f, "<!-- {MAGIC} ]] -->"),
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(super) enum ParseMarkerError {
    #[error("{err}")]
    ParseReplace {
        err: ParseReplaceError,
        #[label]
        span: SourceSpan,
    },
    #[error("no replace specifier found")]
    NoReplace {
        #[label]
        span: SourceSpan,
    },
}

impl From<(ParseReplaceError, SourceSpan)> for ParseMarkerError {
    fn from((err, span): (ParseReplaceError, SourceSpan)) -> Self {
        Self::ParseReplace { err, span }
    }
}

impl Marker {
    pub(super) fn matches(
        text: (&str, SourceSpan),
        manifest: &ManifestFile,
    ) -> Result<Option<Marker>, ParseMarkerError> {
        let Some(body) = Self::matches_marker(text)? else {
            return Ok(None);
        };

        // <replace> [[
        if let Some(replace) = body.strip_suffix_str("[[") {
            let replace = replace.trim();
            let replace = Replace::from_str(replace.0, manifest).map_err(|err| (err, replace.1))?;
            return Ok(Some(Marker::Start(replace)));
        }

        if body.0 == "]]" {
            return Ok(Some(Marker::End));
        }

        let replace = Replace::from_str(body.0, manifest).map_err(|err| (err, body.1))?;
        Ok(Some(Marker::Replace(replace)))
    }

    fn matches_marker(
        text: (&str, SourceSpan),
    ) -> Result<Option<(&str, SourceSpan)>, ParseMarkerError> {
        // <!-- cargo-sync-rdme <body> -->
        let Some(text) = trim_comment(text) else {
            return Ok(None);
        };

        if text.0 == MAGIC {
            return Err(ParseMarkerError::NoReplace { span: text.1 });
        }
        let Some((head, body)) = text.split_once_fn(char::is_whitespace) else {
            return Ok(None);
        };

        Ok((head.0 == MAGIC).then_some(body))
    }
}

fn trim_comment(text: (&str, SourceSpan)) -> Option<(&str, SourceSpan)> {
    let body = text
        .trim()
        .strip_prefix_str("<!--")?
        .trim_start()
        .strip_suffix_str("-->")?
        .trim_end();
    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches() {
        fn ok(s: &str) -> Option<Marker> {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let span = SourceSpan::from(0..s.len());
            Marker::matches((s, span), &manifest).unwrap()
        }
        fn err_kind(s: &str) -> String {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span), &manifest).unwrap_err() {
                ParseMarkerError::ParseReplace {
                    err: ParseReplaceError::UnknownReplace(s),
                    ..
                } => s,
                e => panic!("unexpected: {e}"),
            }
        }
        fn err_norep(s: &str) {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span), &manifest).unwrap_err() {
                ParseMarkerError::NoReplace { .. } => {}
                e => panic!("unexpected: {e}"),
            }
        }

        assert_eq!(ok(""), None);
        assert_eq!(ok("<!-- cargo-sync-rdmexxx -->"), None);

        assert_eq!(
            ok("<!-- cargo-sync-rdme title -->"),
            Some(Marker::Replace(Replace::Title))
        );
        assert!(matches!(
            ok("<!-- cargo-sync-rdme badge [[ -->"),
            Some(Marker::Start(Replace::Badge { name, .. })) if name.is_empty()
        ));
        assert!(matches!(
            ok("<!-- cargo-sync-rdme badge[[-->"),
            Some(Marker::Start(Replace::Badge { name, ..})) if name.is_empty()
        ));
        assert_eq!(ok("<!-- cargo-sync-rdme ]] -->"), Some(Marker::End));

        err_norep("<!-- cargo-sync-rdme  -->");
        assert_eq!(err_kind("<!-- cargo-sync-rdme title [ -->"), "title [");
        assert_eq!(err_kind("<!-- cargo-sync-rdme ] -->"), "]");
    }
}
