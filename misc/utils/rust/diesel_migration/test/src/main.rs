//! A very small binary which diffs the input with the rendered output and fails if they are different.
//! Which allows this to be checked via a bazel test.

use std::collections::BTreeMap;
use std::fmt;

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;
use console::{Style, style};
use similar::{ChangeTag, TextDiff};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "SCHEMA_PATH")]
    schema_path: Utf8PathBuf,

    #[arg(long, env = "SCHEMA_RESULT_PATH")]
    schema_result_path: Utf8PathBuf,

    #[arg(long, env = "SCHEMA_PATCH_PATH")]
    schema_patch_path: Utf8PathBuf,

    #[arg(long, env = "SCHEMA_PATCH_RESULT_PATH")]
    schema_patch_result_path: Utf8PathBuf,
}

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn get_result(path: &std::path::Path) -> anyhow::Result<String> {
    let result = std::fs::read_to_string(path).context("read results")?;
    let result: BTreeMap<Utf8PathBuf, String> = serde_json::from_str(&result).context("parse results")?;
    result.into_values().next().context("get result")
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let runfiles = runfiles::Runfiles::create().expect("failed to create runfiles");
    let schema_path = runfiles::rlocation!(&runfiles, &args.schema_path).expect("failed to get schema file");
    let schema_result_path = runfiles::rlocation!(&runfiles, &args.schema_result_path).expect("failed to get result path");
    let schema_patch_path =
        runfiles::rlocation!(&runfiles, &args.schema_patch_path).expect("failed to get schema patch path");
    let schema_patch_result_path =
        runfiles::rlocation!(&runfiles, &args.schema_patch_result_path).expect("failed to get schema patch result path");

    let schema = std::fs::read_to_string(&schema_path).context("read schema file")?;
    let schema_patch = std::fs::read_to_string(&schema_patch_path).context("read schema patch file")?;
    let schema_result = get_result(&schema_result_path)?;
    let schema_patch_result = get_result(&schema_patch_result_path)?;

    let mut diff_found = false;
    if schema_result != schema {
        println!("Difference found in {}", args.schema_path);
        println!("{}", diff(&schema, &schema_result));
        diff_found = true;
    }
    if schema_patch_result != schema_patch {
        println!("Difference found in {}", args.schema_patch_path);
        println!("{}", diff(&schema_patch, &schema_patch_result));
        diff_found = true;
    }

    if diff_found {
        std::process::exit(1)
    }

    Ok(())
}

pub(crate) fn diff(old: &str, new: &str) -> String {
    use std::fmt::Write;

    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output, "{0:─^1$}┼{0:─^2$}", "─", 9, 120).unwrap();
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    &mut output,
                    "{}{} │{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )
                .unwrap();
                for (_, value) in change.iter_strings_lossy() {
                    write!(&mut output, "{}", s.apply_to(value)).unwrap();
                }
                if change.missing_newline() {
                    writeln!(&mut output).unwrap();
                }
            }
        }
    }

    output
}
