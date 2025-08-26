//! A very small binary which diffs the input with the rendered output and fails if they are different.
//! Which allows this to be checked via a bazel test.

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
    #[arg(long, env = "RENDER_OUTPUT_PATH")]
    render_output: Utf8PathBuf,
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

    let render_output = std::fs::read_to_string(&args.render_output).context("read results")?;
    let render_output: sync_readme_common::SyncReadmeRenderOutput =
        serde_json::from_str(&render_output).context("parse results")?;
    if render_output.rendered == render_output.source {
        println!("readme matches render");
        return Ok(());
    }

    println!("Difference found in {}", render_output.path);

    println!("{}", diff(&render_output.source, &render_output.rendered));

    std::process::exit(1)
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
