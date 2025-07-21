use std::process::ExitStatus;

use rustdoc_types::{Crate, Item};

use crate::{sync::ManifestFile, with_source::{ReadFileError, WithSource}};

mod code_block;
mod heading;
mod intra_link;

type CreateResult<T> = Result<T, CreateRustdocError>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(in super::super) enum CreateRustdocError {
    #[error("failed to create rustdoc process")]
    Spawn(#[source] std::io::Error),
    #[error("rustdoc exited with non-zero status code: {0}")]
    Exit(ExitStatus),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ReadFileError(#[from] ReadFileError),
    #[error("crate {crate_name} does not have a crate-level documentation")]
    RootDocNotFound { crate_name: String },
}

pub(super) fn create(
    app: &App,
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
) -> CreateResult<String> {
    let config = manifest.value().config();

    run_rustdoc(app, package)?;

    let output_file = workspace
        .target_directory
        .join("doc")
        .join(format!("{}.json", package.name.replace('-', "_")));

    let doc_with_source: WithSource<Crate> = WithSource::from_json("rustdoc output", output_file)?;
    let doc = doc_with_source.value();
    let root = doc.index.get(&doc.root).unwrap();
    let local_html_root_url = config.rustdoc.html_root_url.clone().unwrap_or_else(|| {
        format!(
            "https://docs.rs/{}/{}",
            package.name,
            doc.crate_version.as_deref().unwrap_or("latest")
        )
    });

    let root_doc = extract_doc(root)?;
    let mut parser =
        intra_link::Parser::new(doc, root, &local_html_root_url, &config.rustdoc.mappings);
    let events = parser.events(&root_doc);
    let events = heading::convert(events);
    let events = code_block::convert(events);

    let mut buf = String::with_capacity(root_doc.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    Ok(buf)
}

fn run_rustdoc(app: &App, package: &Package) -> CreateResult<()> {
    let mut command = app.toolchain.cargo_command();
    command
        .args(["rustdoc", "--package", &package.name])
        .args(app.feature.cargo_args())
        .args([
            "-Zrustdoc-map",
            "--",
            "--document-private-items",
            "-Zunstable-options",
            "--output-format=json",
        ]);

    tracing::info!(
        "executing {}{}",
        command.get_program().to_string_lossy(),
        command.get_args().fold(String::new(), |mut s, a| {
            s.push(' ');
            s.push_str(a.to_string_lossy().as_ref());
            s
        })
    );

    let status = command.status().map_err(CreateRustdocError::Spawn)?;
    if !status.success() {
        return Err(CreateRustdocError::Exit(status));
    }
    Ok(())
}

fn extract_doc(item: &Item) -> CreateResult<String> {
    item.docs
        .clone()
        .ok_or_else(|| CreateRustdocError::RootDocNotFound {
            crate_name: item.name.clone().unwrap(),
        })
}
