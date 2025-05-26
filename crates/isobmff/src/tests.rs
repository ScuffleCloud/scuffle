#![cfg(test)]
#![cfg_attr(all(test, coverage_nightly), coverage(off))]

use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;

use scuffle_bytes_util::zero_copy::Deserialize;

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

    for (file_name, _) in metadata_file.file_metadata.into_iter().filter(|(_, m)| m.published) {
        println!("testing {file_name}");
        let mut file = std::fs::File::open(dir.join("files").join(&file_name)).expect("failed to open file");
        let reader = scuffle_bytes_util::zero_copy::IoRead::from(BufReader::new(&mut file));

        let isobmff_file = IsobmffFile::deserialize(reader).expect("failed to deserialize file");
        assert_eq!(
            isobmff_file.size(),
            file.metadata().expect("failed to read metadata").len() as usize,
            "file size mismatch",
        );
    }
}
