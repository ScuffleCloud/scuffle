use crate::types::ProtoPath;
use std::iter;

#[derive(Clone, Debug, Default)]
pub(crate) struct PathSet {
    paths: Vec<String>,
}

impl PathSet {
    pub(crate) fn contains(&self, full_path: &ProtoPath) -> bool {
        let mut path_str = full_path.to_string();
        if !path_str.starts_with(".") {
            path_str = format!(".{}", path_str).to_string();
        }
        self.find_matching(&path_str).is_some()
    }

    pub(crate) fn insert(&mut self, path: &str) {
        self.paths.push(path.to_string())
    }

    fn find_matching(&self, full_path: &str) -> Option<&String> {
        sub_path_iter(full_path).find_map(|path| self.paths.iter().find(|p| *p == path))
    }
}

fn sub_path_iter(full_path: &str) -> impl Iterator<Item = &str> {
    // Get all combinations of prefixes/suffixes, along global path
    iter::once(full_path)
        .chain(suffixes(full_path))
        .chain(prefixes(full_path))
        .chain(iter::once("."))
}

fn prefixes(fq_path: &str) -> impl Iterator<Item = &str> {
    iter::successors(Some(fq_path), |path| {
        #[allow(unknown_lints, clippy::manual_split_once)]
        path.rsplitn(2, '.').nth(1).filter(|path| !path.is_empty())
    })
    .skip(1)
}

fn suffixes(fq_path: &str) -> impl Iterator<Item = &str> {
    iter::successors(Some(fq_path), |path| {
        #[allow(unknown_lints, clippy::manual_split_once)]
        path.splitn(2, '.').nth(1).filter(|path| !path.is_empty())
    })
    .skip(1)
}

#[cfg(test)]
#[cfg(feature = "prost")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::PathSet;
    use crate::types::ProtoPath;

    #[test]
    fn test_path_set() {
        let mut ps_a = PathSet::default();
        ps_a.insert(".my_package.MessageA.field_a");

        assert!(ps_a.contains(&ProtoPath::new(".my_package.MessageA.field_a")));
        assert!(!ps_a.contains(&ProtoPath::new(".my_package.MessageA.field_b")));
        assert!(!ps_a.contains(&ProtoPath::new(".my_package.MessageB.field_a")));
        assert!(!ps_a.contains(&ProtoPath::new(".other_package.MessageA.field_a")));

        let mut ps_b = PathSet::default();
        ps_b.insert(".my_package.MessageA");

        assert!(ps_b.contains(&ProtoPath::new(".my_package.MessageA.field_a")));
        assert!(ps_b.contains(&ProtoPath::new(".my_package.MessageA.field_b")));
        assert!(!ps_b.contains(&ProtoPath::new(".my_package.MessageB.field_a")));
        assert!(!ps_b.contains(&ProtoPath::new(".other_package.MessageA.field_a")));

        let mut ps_c = PathSet::default();
        ps_c.insert(".my_package");

        assert!(ps_c.contains(&ProtoPath::new(".my_package.MessageA.field_a")));
        assert!(ps_c.contains(&ProtoPath::new(".my_package.MessageA.field_b")));
        assert!(ps_c.contains(&ProtoPath::new(".my_package.MessageB.field_a")));
        assert!(!ps_c.contains(&ProtoPath::new(".other_package.MessageA.field_a")));

        let mut ps_d = PathSet::default();
        ps_d.insert(".");

        assert!(ps_d.contains(&ProtoPath::new(".my_package.MessageA.field_a")));
        assert!(ps_d.contains(&ProtoPath::new(".my_package.MessageA.field_b")));
        assert!(ps_d.contains(&ProtoPath::new(".my_package.MessageB.field_a")));
        assert!(ps_d.contains(&ProtoPath::new(".other_package.MessageA.field_a")));
    }
}
