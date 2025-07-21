use std::collections::BTreeMap;
use std::io::Write;
use std::process::Stdio;

use camino::Utf8PathBuf;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    rustc: Utf8PathBuf,

    #[arg(long)]
    out_dir: Utf8PathBuf,

    #[arg(long)]
    extracted_tests: Utf8PathBuf,

    #[arg(long)]
    edition: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    extra: Vec<String>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct RustDocExtract {
    format_version: i32,
    doctests: Vec<ExtractedDocTest>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct ExtractedDocTest {
    file: String,
    line: i32,
    doctest_attributes: DoctestAttributes,
    doctest_code: String,
    name: String,
}

#[derive(Debug, serde_derive::Deserialize)]
struct DoctestAttributes {
    should_panic: bool,
    no_run: bool,
    ignore: String,
    rust: bool,
    test_harness: bool,
    compile_fail: bool,
    standalone_crate: bool,
    error_codes: Vec<String>,
    edition: Option<String>,
}

fn make_test_case(name: &str, ignore: bool, file: &str, line: usize, should_panic: bool, func_path: &str) -> String {
    format!(
        r#"&const {{
    test::TestDescAndFn::new_doctest(
        "{name}",
        {ignore},
        "{file}",
        {line},
        false,
        {should_panic},
        test::StaticTestFn(
            #[coverage(off)]
            || test::assert_test_result({func_path}()),
        ),
    )
}}"#
    )
}

fn test_binary(extern_crates: &str, test_cases: &[String]) -> String {
    let test_cases = test_cases.join(",");
    format!(
        "
#![feature(test)]
#![feature(coverage_attribute)]

extern crate test;
{extern_crates}
        
#[doc(hidden)]
#[coverage(off)]
fn main() {{
    test::test_main_static(&[{test_cases}])
}}"
    )
}

fn test_function(test_ident: &str, code: &str) -> String {
    format!(
        "pub mod {test_ident} {{
    {code}

    pub fn __main_fn() -> impl std::process::Termination {{
        main()
    }}
}}"
    )
}

struct Dirs {
    src: Utf8PathBuf,
    rlib: Utf8PathBuf,
    compile_fail: Utf8PathBuf,
    bin: Utf8PathBuf,
}

fn compile_merged(
    args: &Args,
    all_tests: &[ExtractedDocTest],
    dirs: &Dirs,
    edition_merged_tests: BTreeMap<&str, Vec<(usize, String)>>,
) -> Utf8PathBuf {
    struct MergedLib {
        edition: String,
        filename: Utf8PathBuf,
        crate_ident: String,
    }

    let mut all_test_cases = Vec::new();
    let mut externs = String::new();
    let merged_files: Vec<_> = edition_merged_tests
        .iter()
        .map(|(edition, tests)| {
            let crate_ident = format!("doctest_merged_{edition}");
            let tokens = tests
                .iter()
                .map(|(idx, content)| test_function(&format!("test_{idx}"), content))
                .collect::<Vec<_>>();
            all_test_cases.extend(tests.iter().filter_map(|(idx, _)| {
                let test: &ExtractedDocTest = &all_tests[*idx];
                if !test.doctest_attributes.no_run {
                    Some(make_test_case(
                        &test.name,
                        test.doctest_attributes.ignore != "None",
                        &test.file,
                        test.line as usize,
                        test.doctest_attributes.should_panic,
                        &format!("{crate_ident}::test_{idx}::__main_fn"),
                    ))
                } else {
                    None
                }
            }));

            let filename = dirs.src.join(format!("{crate_ident}.rs"));
            std::fs::write(&filename, tokens.join("\n\n")).expect("failed to write merged");

            externs.push_str("extern crate ");
            externs.push_str(&crate_ident);
            externs.push_str(";\n");

            MergedLib {
                edition: edition.to_string(),
                filename,
                crate_ident,
            }
        })
        .collect();

    let merged_binary = test_binary(&externs, &all_test_cases);
    let rustdoc_merged_file = dirs.src.join("rustdoc_merged.rs");
    std::fs::write(&rustdoc_merged_file, merged_binary).expect("failed to write merged");

    for file in &merged_files {
        let status = std::process::Command::new(&args.rustc)
            .args(&args.extra)
            .arg("--crate-type=rlib")
            .arg("--crate-name")
            .arg(&file.crate_ident)
            .arg("--edition")
            .arg(&file.edition)
            .arg(&file.filename)
            .arg("--out-dir")
            .arg(&dirs.rlib)
            .status()
            .expect("failed to compile");

        if !status.success() {
            std::process::exit(status.code().unwrap_or(127))
        }
    }

    let mut binary_out = dirs.bin.join("rustdoc_merged");
    if cfg!(windows) {
        binary_out.set_extension("exe");
    }

    let status = std::process::Command::new(&args.rustc)
        .args(&args.extra)
        .arg(format!("-L{}", dirs.rlib))
        .arg("--crate-type=bin")
        .arg("--crate-name")
        .arg("rustdoc_merged")
        .arg("--edition")
        .arg("2024")
        .arg(&rustdoc_merged_file)
        .arg("-o")
        .arg(&binary_out)
        .env("RUSTC_BOOTSTRAP", "1")
        .status()
        .expect("failed to compile");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(127))
    }

    binary_out
}

fn standalone_test_runner(env_var: &str) -> String {
    format!(
        r#"fn main() {{
    if let Some(binary) = std::env::var_os("{env_var}") {{
        let status = ::std::process::Command::new(binary)
            .status()
            .expect("failed to run test binary");
        if !status.success() {{
            panic!("test exited with {{status:?}}")
        }}
    }} else {{
        panic!("{env_var} is not set");
    }}
}}"#
    )
}

fn compile_fail_test_result(error: Option<&str>) -> String {
    let Some(error) = error else {
        return "fn main() {}".into();
    };

    format!(
        r#"fn main() {{
    panic!("{error}");
}}"#
    )
}

fn main() {
    let args = Args::parse();

    let extracted_tests = std::fs::read_to_string(&args.extracted_tests).expect("failed to read extracted tests");

    let mut all_tests = Vec::new();

    for line in extracted_tests.lines() {
        let extract: RustDocExtract = serde_json::from_str(line).expect("invalid doctest output");
        if extract.format_version != 1 {
            panic!("format version mismatch: 1 != {}", extract.format_version);
        }

        all_tests.extend(extract.doctests);
    }

    let mut standalone_tests = Vec::new();
    let mut edition_merged_tests: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut compile_fail_tests = Vec::new();
    let mut test_harness_tests = Vec::new();

    let dirs = Dirs {
        bin: args.out_dir.join("bin"),
        src: args.out_dir.join("src"),
        compile_fail: args.out_dir.join("compile_fail"),
        rlib: args.out_dir.join("rlib"),
    };

    std::fs::create_dir_all(&dirs.bin).expect("failed to create bin dir");
    std::fs::create_dir_all(&dirs.src).expect("failed to create src dir");
    std::fs::create_dir_all(&dirs.compile_fail).expect("failed to create compile_fail dir");
    std::fs::create_dir_all(&dirs.rlib).expect("failed to create rlib dir");

    for (idx, test) in all_tests.iter().enumerate() {
        if !test.doctest_attributes.rust {
            continue;
        }

        if !test.doctest_attributes.standalone_crate
            && !test.doctest_attributes.compile_fail
            && !test.doctest_attributes.test_harness
        {
            edition_merged_tests
                .entry(test.doctest_attributes.edition.as_ref().unwrap_or(&args.edition).as_str())
                .or_default()
                .push((idx, test.doctest_code.clone()));
        } else {
            match (test.doctest_attributes.compile_fail, test.doctest_attributes.test_harness) {
                (true, _) => compile_fail_tests.push(idx),
                (false, true) => test_harness_tests.push(idx),
                (false, false) => standalone_tests.push(idx),
            }
        }
    }

    for idx in &compile_fail_tests {
        let crate_name = format!("compile_fail_{idx}");
        let test = &all_tests[*idx];
        let mut child = std::process::Command::new(&args.rustc)
            .args(&args.extra)
            .arg("--test")
            .arg("--crate-type=rlib")
            .arg("--crate-name")
            .arg(crate_name)
            .arg("--edition")
            .arg(test.doctest_attributes.edition.as_ref().unwrap_or(&args.edition))
            .arg("-")
            .arg("--out-dir")
            .arg(&dirs.compile_fail)
            .env("UNSTABLE_RUSTDOC_TEST_PATH", &test.file)
            .env("UNSTABLE_RUSTDOC_TEST_LINE", test.line.to_string())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to compile");

        let mut stdin = child.stdin.take().unwrap();

        stdin.write_all(test.doctest_code.as_bytes()).expect("failed to write stdin");
        drop(stdin);

        let output = child.wait_with_output().expect("failed to wait for output");

        if output.status.success() {
            edition_merged_tests.entry("2024").or_default().push((
                *idx,
                compile_fail_test_result(Some("test marked as compile_fail but successfully compiled")),
            ))
        } else {
            let stderr = String::from_utf8(output.stderr).expect("bad stderr");
            let missing_ecs: Vec<_> = test
                .doctest_attributes
                .error_codes
                .iter()
                .filter(|ec| !stderr.contains(*ec))
                .collect();
            if missing_ecs.is_empty() {
                edition_merged_tests
                    .entry("2024")
                    .or_default()
                    .push((*idx, compile_fail_test_result(None)))
            } else {
                edition_merged_tests.entry("2024").or_default().push((
                    *idx,
                    compile_fail_test_result(Some(&format!("Some expected error codes were not found: {missing_ecs:?}"))),
                ))
            }
        }
    }

    let mut standalone_binary_envs = Vec::new();

    for idx in &standalone_tests {
        let test = &all_tests[*idx];
        let crate_name = format!("standalone_{idx}");
        let mut binary_out = dirs.bin.join(&crate_name);
        if cfg!(windows) {
            binary_out.set_extension("exe");
        }

        let mut child = std::process::Command::new(&args.rustc)
            .args(&args.extra)
            .arg("--crate-type=bin")
            .arg("--crate-name")
            .arg(crate_name)
            .arg("--edition")
            .arg(all_tests[*idx].doctest_attributes.edition.as_ref().unwrap_or(&args.edition))
            .arg("-")
            .arg("-o")
            .arg(&binary_out)
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .env("UNSTABLE_RUSTDOC_TEST_PATH", &test.file)
            .env("UNSTABLE_RUSTDOC_TEST_LINE", test.line.to_string())
            .spawn()
            .expect("failed to compile");

        let mut stdin = child.stdin.take().unwrap();

        stdin.write_all(test.doctest_code.as_bytes()).expect("failed to write stdin");
        drop(stdin);

        let status = child.wait().expect("failed to wait for output");

        if !status.success() {
            std::process::exit(status.code().unwrap_or(127))
        }

        let env_var = format!("RUSTDOC_TEST_{idx}_BINARY");
        edition_merged_tests
            .entry("2024")
            .or_default()
            .push((*idx, standalone_test_runner(&env_var)));
        standalone_binary_envs.push((env_var, binary_out.strip_prefix(&args.out_dir).unwrap().to_owned()));
    }

    let mut test_binaries = Vec::new();

    // Compile all tests which are merged.
    if !edition_merged_tests.is_empty() {
        test_binaries.push(rust_doctest_common::TestBinary {
            name: "Merged Doctests".to_string(),
            path: compile_merged(&args, &all_tests, &dirs, edition_merged_tests)
                .strip_prefix(&args.out_dir)
                .unwrap()
                .to_owned(),
        });
    }

    for idx in &test_harness_tests {
        let test = &all_tests[*idx];
        let crate_name = format!("test_harness_{idx}");
        let mut binary_out = dirs.bin.join(&crate_name);
        if cfg!(windows) {
            binary_out.set_extension("exe");
        }

        let mut child = std::process::Command::new(&args.rustc)
            .args(&args.extra)
            .arg("--test")
            .arg("--crate-type=bin")
            .arg("--crate-name")
            .arg(crate_name)
            .arg("--edition")
            .arg(all_tests[*idx].doctest_attributes.edition.as_ref().unwrap_or(&args.edition))
            .arg("-")
            .arg("-o")
            .arg(&binary_out)
            .env("UNSTABLE_RUSTDOC_TEST_PATH", &test.file)
            .env("UNSTABLE_RUSTDOC_TEST_LINE", test.line.to_string())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("failed to compile");

        let mut stdin = child.stdin.take().unwrap();

        stdin.write_all(test.doctest_code.as_bytes()).expect("failed to write stdin");
        drop(stdin);

        let status = child.wait().expect("failed to wait for output");

        if !status.success() {
            std::process::exit(status.code().unwrap_or(127))
        }

        test_binaries.push(rust_doctest_common::TestBinary {
            name: test.name.clone(),
            path: binary_out.strip_prefix(&args.out_dir).unwrap().to_owned(),
        });
    }

    let manifest = serde_json::to_string_pretty(&rust_doctest_common::Manifest {
        test_binaries,
        standalone_binary_envs,
    })
    .unwrap();

    std::fs::write(args.out_dir.join("manifest.json"), manifest).expect("failed to write manifest");
}
