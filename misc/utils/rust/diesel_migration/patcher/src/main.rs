use std::collections::BTreeMap;
use std::io::Write;
use std::process::Stdio;

use camino::Utf8PathBuf;
use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, env = "OUTPUT_FILE")]
    output_file: Utf8PathBuf,

    #[clap(long, env = "SCHEMA_FILE")]
    schema_file: Utf8PathBuf,

    #[clap(long, env = "TEMP_DIR")]
    temp_dir: Utf8PathBuf,

    #[clap(long, env = "SCHEMA_PATCH_FILE")]
    schema_patch_file: Utf8PathBuf,

    #[clap(long, env = "SCHEMA_FILE_RESULTS")]
    schema_file_results: Utf8PathBuf,

    #[clap(long, env = "PATCH_BINARY")]
    patch_binary: Utf8PathBuf,

    #[clap(long, env = "DIFF_BINARY")]
    diff_binary: Utf8PathBuf,
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    std::fs::create_dir_all(&args.temp_dir).expect("failed to create temp dir");

    let results = std::fs::read_to_string(&args.schema_file_results).expect("failed to read results");
    let results: BTreeMap<Utf8PathBuf, String> = serde_json::from_str(&results).expect("failed to parse results");
    let unedited_file = results.get(&args.schema_file).expect("failed to get unedited file").clone();

    let unedited = args.temp_dir.join("unedited.rs");

    std::fs::write(&unedited, &unedited_file).expect("failed to write unedited file");

    // So the unedited file has been patched via the patch file, so we need to first reverse the patch
    // and then generate a new diff file and output it to the output file
    let reverse_patch = std::process::Command::new(args.patch_binary)
        .arg("--follow-symlinks")
        .arg("-R")
        .arg("-i")
        .arg(&args.schema_patch_file)
        .arg("-o")
        .arg("-")
        .arg(&unedited)
        .output()
        .expect("failed to reverse patch");

    if !reverse_patch.status.success() {
        std::io::stderr()
            .write_all(&reverse_patch.stderr)
            .expect("failed to write stderr");
        std::io::stderr()
            .write_all(&reverse_patch.stdout)
            .expect("failed to write stdout");
        panic!("failed to reverse patch");
    }

    let reverse_patch_output = String::from_utf8(reverse_patch.stdout).expect("failed to read reverse patch output");

    let mut diff = std::process::Command::new(args.diff_binary)
        .arg("-u")
        .arg(format!("--label=a/{}", args.schema_file))
        .arg(format!("--label=b/{}", args.schema_file.with_extension("unpatched.rs")))
        .arg("-")
        .arg(&args.schema_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to generate diff");

    let mut stdin = diff.stdin.take().unwrap();
    std::thread::spawn(move || {
        stdin
            .write_all(reverse_patch_output.as_bytes())
            .expect("failed to write stdin");
    });

    let output = diff.wait_with_output().expect("failed to wait for output");
    if output.status.code().is_none_or(|c| c != 0 && c != 1) {
        std::io::stderr().write_all(&output.stderr).expect("failed to write stderr");
        std::io::stderr().write_all(&output.stdout).expect("failed to write stdout");
        panic!("failed to generate diff");
    }

    let content = String::from_utf8(output.stdout).expect("failed to convert stdout to utf8");
    let json = serde_json::to_string(&BTreeMap::from_iter([(args.schema_patch_file, content)]))
        .expect("failed to serialize output");
    std::fs::write(args.output_file, json).expect("failed to write diff");
}
