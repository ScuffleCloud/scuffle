//! This is a binary helper to merge rustdoc html outputs
//!
//! Normally rustdoc is run sequentially outputting to a single output directory. This goes against how
//! bazel works and causes issues since every time we change one crate we would need to regenerate the docs for all crates.
//!
//! This tool fixes this problem by essentially performing the same steps that rustdoc would do when merging except does it
//! in a bazel-like way.

use std::path::Path;
use std::process::Stdio;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    rustdoc: Utf8PathBuf,
    #[arg(long)]
    manifest: Utf8PathBuf,
    #[arg(long)]
    output: Utf8PathBuf,
}

#[derive(Debug, serde_derive::Deserialize)]
struct Manifest {
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct ManifestEntry {
    html_out: Option<Utf8PathBuf>,
    parts_out: Option<Utf8PathBuf>,
    json_out: Option<Utf8PathBuf>,
    crate_name: String,
    crate_version: String,
}

fn copy(input: &Utf8Path, output: &Utf8Path, make_path: impl Fn(&Utf8Path) -> Utf8PathBuf) {
    let input = make_path(input);
    let output = make_path(output);
    copy_dir::copy_dir(input, output).expect("failed to copy");
}

fn path_depth(mut path: &Path) -> usize {
    let mut count = 0;
    while let Some(parent) = path.parent() {
        path = parent;
        count += 1;
    }
    count
}

fn process_file(target: &Path, file: &Path, replacements: &[String]) -> std::io::Result<()> {
    if file
        .extension()
        .and_then(|ext| ext.to_str())
        .is_none_or(|ext| !matches!(ext, "css" | "js" | "html"))
    {
        return Ok(());
    }

    let content = std::fs::read(file)?;
    let Ok(content_str) = String::from_utf8(content) else {
        return Ok(());
    };

    let depth = path_depth(file.parent().unwrap().strip_prefix(target).unwrap());
    let mut prefix = String::new();
    for _ in 0..depth {
        prefix.push_str("../");
    }

    let mut prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        prefix = "."
    }

    let mut changed = content_str.clone();
    for replace in replacements {
        changed = changed.replace(replace, prefix);
    }

    if changed != content_str {
        std::fs::remove_file(file)?;
        std::fs::write(file, changed)?;
    }

    Ok(())
}

fn walk_and_replace(target: &Path, replacements: &[String]) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(target)?.collect();
    while let Some(entry) = entries.pop() {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            entries.extend(std::fs::read_dir(entry.path())?);
        } else if file_type.is_file() {
            process_file(target, &entry.path(), replacements)?;
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    let rustdoc_version = std::process::Command::new(&args.rustdoc)
        .arg("--version")
        .output()
        .expect("rustdoc version failed");

    if !rustdoc_version.status.success() {
        panic!(
            "failed to get rustdoc version {}",
            String::from_utf8_lossy(&rustdoc_version.stderr)
        )
    }

    let Ok(output) = String::from_utf8(rustdoc_version.stdout) else {
        panic!("invalid utf8 from rustdoc --version");
    };

    let splits: Vec<_> = output.splitn(3, ' ').collect();
    if splits.len() != 3 {
        panic!("invalid rustdoc output: {output}")
    }

    let tmp_dir = args.output.join(".tmp");

    let status = std::process::Command::new(&args.rustdoc)
        .env("RUSTC_BOOTSTRAP", "1")
        .arg("-Zunstable-options")
        .arg("--enable-index-page")
        .arg("--out-dir")
        .arg(&tmp_dir)
        .arg("-")
        .stdin(Stdio::null())
        .status()
        .expect("failed to run rustdoc");

    if !status.success() {
        panic!("failed to generate rustdoc")
    }

    let vars_edit = || {
        element!(r#"meta[name="rustdoc-vars"]"#, |el| {
            el.set_attribute("data-current-crate", "")?;
            Ok(())
        })
    };

    let manifest = std::fs::read_to_string(&args.manifest).expect("manifest read");
    let manifest: Manifest = serde_json::from_str(&manifest).expect("manifest deserializse");

    let mut sorted_names: Vec<_> = manifest
        .entries
        .iter()
        .filter(|e| e.parts_out.is_some())
        .map(|e| &e.crate_name)
        .collect();
    sorted_names.sort();

    let index_page = rewrite_str(
        &std::fs::read_to_string(tmp_dir.join("index.html")).expect("index.html missing"),
        RewriteStrSettings {
            element_content_handlers: vec![
                element!("ul.all-items", |el| {
                    el.set_inner_content("", ContentType::Html);
                    for name in &sorted_names {
                        el.append(
                            &format!(r#"<li><a href="{name}/index.html">{name}</a></li>"#),
                            ContentType::Html,
                        );
                    }
                    Ok(())
                }),
                vars_edit(),
            ],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();
    let help_page = rewrite_str(
        &std::fs::read_to_string(tmp_dir.join("help.html")).expect("help.html missing"),
        RewriteStrSettings {
            element_content_handlers: vec![vars_edit()],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();
    let settings_page = rewrite_str(
        &std::fs::read_to_string(tmp_dir.join("settings.html")).expect("settings.html missing"),
        RewriteStrSettings {
            element_content_handlers: vec![vars_edit()],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();

    std::fs::create_dir_all(&args.output).expect("create output");
    std::fs::write(args.output.join("index.html"), index_page).expect("index.html write");
    std::fs::write(args.output.join("help.html"), help_page).expect("help.html write");
    std::fs::write(args.output.join("settings.html"), settings_page).expect("settings.html write");

    std::fs::create_dir_all(args.output.join("search.desc")).expect("make dir");
    std::fs::create_dir_all(args.output.join("src")).expect("make dir");

    let mut finalize_cmd = std::process::Command::new(&args.rustdoc);
    finalize_cmd
        .env("RUSTC_BOOTSTRAP", "1")
        .arg("-Zunstable-options")
        .arg("--merge=finalize")
        .arg("-o")
        .arg(&args.output);

    let mut replacements = Vec::new();

    for entry in &manifest.entries {
        let crate_name = entry.crate_name.replace("-", "_");
        if let (Some(parts_out), Some(html_out)) = (&entry.parts_out, &entry.html_out) {
            finalize_cmd.arg("--include-parts-dir").arg(parts_out);
            copy(html_out, &args.output, |path| path.join(&crate_name));
            copy(html_out, &args.output, |path| path.join("src").join(&crate_name));
            copy(html_out, &args.output, |path| path.join("search.desc").join(&crate_name));
            replacements.push(format!("https://docs.rs/{crate_name}/{}", entry.crate_version))
        } else if let Some(json_out) = &entry.json_out {
            std::fs::copy(json_out, args.output.join(format!("{crate_name}.json"))).expect("failed to copy json output");
        }
    }

    let status = finalize_cmd.status().expect("failed to run finalize");
    if !status.success() {
        panic!("failed to run rustdoc finalize");
    }

    walk_and_replace(args.output.as_std_path(), &replacements).expect("failed to replace paths")
}
