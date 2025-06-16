fn main() {
    let mut config = tinc_build::Config::prost();

    if let Some(pre_compiled_fds) = std::env::var_os("TINC_SCUFFLE_CLOUD_COMPILED_FD") {
        let fds = std::fs::read(pre_compiled_fds).expect("pre_compiled_fds not found");
        config.load_fds(fds.as_slice())
    } else {
        let mut files = Vec::new();
        for file in glob::glob("pb/**/*.proto").expect("glob failed") {
            files.push(file.expect("bad file"));
        }

        config.compile_protos(&files, &["pb"])
    }
    .expect("compile failed")
}
