#![cfg(test)]
#![cfg_attr(all(test, coverage_nightly), coverage(off))]

use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize};

use crate::{IsoSized, IsobmffFile};

#[derive(Debug, serde_derive::Deserialize)]
struct MetadataFile {
    file_metadata: BTreeMap<String, FileMetadata>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct FileMetadata {
    published: bool,
}

#[test]
fn conformance_files() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join("isobmff_conformance");

    let metadata_file: MetadataFile =
        serde_json::from_reader(std::fs::File::open(dir.join("files.json")).expect("failed to open metadata file"))
            .expect("failed to deserialize metadata file");

    for (file_name, _) in metadata_file
        .file_metadata
        .into_iter()
        .filter(|(n, m)| !n.ends_with(".zip") && m.published)
    {
        println!("testing {file_name}");
        let mut file = std::fs::File::open(dir.join("files").join(&file_name)).expect("failed to open file");
        let reader = scuffle_bytes_util::zero_copy::IoRead::from(BufReader::new(&mut file));

        let isobmff_file = IsobmffFile::deserialize(reader).expect("failed to deserialize file");

        let file_size = file.metadata().expect("failed to read metadata").len() as usize;
        assert!(
            isobmff_file.size() <= file_size,
            "file size mismatch: {} (serialized size) > {} (file size)",
            isobmff_file.size(),
            file_size
        );

        let mut serialized = Vec::new();
        isobmff_file.serialize(&mut serialized).expect("failed to serialize file");
        std::fs::write(
            dir.join("output").join(file_name.split('/').last().unwrap_or(&file_name)),
            &serialized,
        )
        .expect("failed to write serialized file");
        let redeserialized_file = IsobmffFile::deserialize(scuffle_bytes_util::zero_copy::Slice::from(&serialized[..]))
            .expect("failed to deserialize serialized file");
        assert_eq!(isobmff_file, redeserialized_file, "file content mismatch");
    }
}
