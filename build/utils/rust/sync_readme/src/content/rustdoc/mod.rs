use anyhow::Context;
use rustdoc_types::{Crate, Item};

use crate::config::Package;

mod code_block;
mod heading;
mod intra_link;

pub(super) fn create(package: &Package) -> anyhow::Result<String> {
    let doc_string = std::fs::read_to_string(&package.rustdoc_json).context("read rustdoc")?;
    let doc: Crate = serde_json::from_str(&doc_string).context("parse rustdoc")?;
    let root = doc.index.get(&doc.root).unwrap();
    let local_html_root_url = package.metadata.rustdoc_html_root_url.clone().unwrap_or_else(|| {
        format!(
            "https://docs.rs/{}/{}",
            package.name,
            doc.crate_version.as_ref().unwrap_or(&package.version)
        )
    });

    let root_doc = extract_doc(root);
    let mut parser = intra_link::Parser::new(&doc, root, &local_html_root_url, &package.metadata.rustdoc_mappings);
    let events = parser.events(&root_doc);
    let events = heading::convert(events);
    let events = code_block::convert(events);

    let mut buf = String::with_capacity(root_doc.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();

    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }

    Ok(buf)
}

fn extract_doc(item: &Item) -> String {
    item.docs.clone().unwrap_or_default()
}
