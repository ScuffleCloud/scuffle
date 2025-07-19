use camino::Utf8PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "CLIPPY_OUTPUTS", value_delimiter = ';')]
    outputs: Vec<Utf8PathBuf>,
}

#[derive(serde_derive::Deserialize)]
struct ClippyJson {
    #[serde(rename = "$message_type")]
    message_type: String,
    rendered: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut failed = false;
    for manifest in args.outputs {
        let manifest = std::fs::read_to_string(manifest).expect("CLIPPY_OUTPUTS malformed");
        for line in manifest.lines().map(|line| line.trim()).filter(|line| !line.is_empty()) {
            let diagnostic = serde_json::from_str::<ClippyJson>(line).expect("DIAGNOSTIC malformed");
            if diagnostic.message_type == "diagnostic" {
                failed |= true;
                if let Some(rendered) = diagnostic.rendered {
                    eprintln!("{rendered}")
                }
            }
        }
    }

    std::process::exit(if failed { 1 } else { 0 })
}
