use camino::Utf8PathBuf;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    args_file: Utf8PathBuf,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    extra: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let args_file_content = serde_json::to_string_pretty(&args.extra).expect("malformed args");

    if let Some(parent) = args.args_file.parent() {
        std::fs::create_dir_all(parent).expect("cannot create directory")
    }

    std::fs::write(args.args_file, args_file_content).expect("unable to write to file")
}
