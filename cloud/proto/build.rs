fn main() {
    let mut files = Vec::new();
    for file in glob::glob("pb/**/*.proto").expect("glob failed") {
        files.push(file.expect("bad file"))
    }

    tinc_build::Config::prost()
        .compile_protos(&files, &["pb"])
        .expect("compile failed")
}
