use std::fmt;

use super::{marker::Replace, ManifestFile};

mod badge;
mod rustdoc;
mod title;

pub(super) fn create_all(
    replaces: impl IntoIterator<Item = Replace>,
    app: &App,
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
) -> Result<Vec<Contents>, CreateAllContentsError> {
    let mut contents = vec![];
    let mut errors = vec![];
    for replace in replaces {
        let res = replace.create_content(app, manifest, workspace, package);
        match res {
            Ok(c) => contents.push(c),
            Err(err) => errors.push(err),
        }
    }

    if !errors.is_empty() {
        return Err(CreateAllContentsError { errors });
    }

    Ok(contents)
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("failed to create contents of README")]
pub(super) struct CreateAllContentsError {
    #[related]
    errors: Vec<CreateContentsError>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(super) enum CreateContentsError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    CreateBadge(#[from] badge::CreateAllBadgesError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    CreateRustdoc(#[from] rustdoc::CreateRustdocError),
}

#[derive(Debug, Clone)]
pub(super) struct Contents {
    text: String,
}

impl Replace {
    fn create_content(
        self,
        app: &App,
        manifest: &ManifestFile,
        workspace: &Metadata,
        package: &Package,
    ) -> Result<Contents, CreateContentsError> {
        let text = match self {
            Replace::Title => title::create(package),
            Replace::Badge { name: _, badges } => {
                badge::create_all(badges, manifest, workspace, package)?
            }
            Replace::Rustdoc => rustdoc::create(app, manifest, workspace, package)?,
        };

        assert!(text.is_empty() || text.ends_with('\n'));

        Ok(Contents { text })
    }
}

impl Contents {
    pub(super) fn text(&self) -> &str {
        &self.text
    }
}

struct Escape<'s>(&'s str, &'s [char]);

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0;
        while let Some(idx) = s.find(self.1) {
            f.write_str(&s[..idx])?;
            write!(f, r"\{}", s.as_bytes()[idx] as char)?;
            s = &s[idx + 1..];
        }
        f.write_str(s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape() {
        let need_escape = [
            '\\', '`', '*', '_', '{', '}', '[', ']', '(', ')', '>', '#', '+', '-', '.', '!',
        ];

        assert_eq!(Escape(r"foo", &need_escape).to_string(), r"foo");
        assert_eq!(Escape(r"`foobar", &need_escape).to_string(), r"\`foobar");
        assert_eq!(Escape(r"foo*bar", &need_escape).to_string(), r"foo\*bar");
        assert_eq!(Escape(r"foobar_", &need_escape).to_string(), r"foobar\_");
        assert_eq!(
            Escape(r"`foo*bar_", &need_escape).to_string(),
            r"\`foo\*bar\_"
        );
        assert_eq!(
            Escape(r"\foo\bar\", &need_escape).to_string(),
            r"\\foo\\bar\\"
        );
        assert_eq!(Escape(r"*", &need_escape).to_string(), r"\*");
    }
}
