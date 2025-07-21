use camino::Utf8PathBuf;

/// A binary that has a test harness that is like libtest
#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestBinary {
    /// The name of this binary.
    pub name: String,
    /// Relative path to the binary relative to the rust_doctest_builder output directory.
    pub path: Utf8PathBuf,
}

/// A manifest created by the rust_doctest_builder, used
/// by the rust_doctest_runner.
#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// A list of binaries created.
    pub test_binaries: Vec<TestBinary>,
    /// Some tests are standalone binaries these are found via ENV vars.
    /// These paths are all relative to the rust_doctest_builder output directory.
    pub standalone_binary_envs: Vec<(String, Utf8PathBuf)>,
}
