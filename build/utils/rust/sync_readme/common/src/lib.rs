//! Some common outputs for the sync-readme utilities.

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct SyncReadmeRenderOutput {
    pub source: String,
    pub rendered: String,
    pub path: camino::Utf8PathBuf,
}
