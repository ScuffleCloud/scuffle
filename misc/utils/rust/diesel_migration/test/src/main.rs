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
    #[arg(long, env = "RESULT_PATH")]
    result_path: Utf8PathBuf,

    #[arg(long, env = "SCHEMAS_PATHS", value_delimiter = ';')]
    schemas_paths: Vec<Utf8PathBuf>,

    #[arg(long, env = "SCHEMAS_SHORT_PATHS", value_delimiter = ';')]
    schemas_short_paths: Vec<Utf8PathBuf>,
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let runfiles = runfiles::Runfiles::create().expect("failed to create runfiles");
    let path_map = args
        .schemas_paths
        .iter()
        .zip(args.schemas_short_paths.iter())
        .collect::<BTreeMap<_, _>>();
    let result_path = runfiles::rlocation!(&runfiles, args.result_path).expect("failed to get result path");

    let results = std::fs::read_to_string(&result_path).context("read results")?;
    let results: BTreeMap<Utf8PathBuf, String> = serde_json::from_str(&results).context("parse results")?;

    let mut diff_found = false;
    for (path, content) in results {
        let actual_path = runfiles::rlocation!(&runfiles, path_map[&path]).expect("failed to get actual path");
        let expected = std::fs::read_to_string(&actual_path).context("read expected")?;

        if content != expected {
            diff_found = true;
            println!("Difference found in {}", path);
            println!("{}", diff(&expected, &content));
        }
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
