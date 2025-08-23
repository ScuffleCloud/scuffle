use crate::config::Package;

pub(super) fn create(package: &Package) -> String {
    format!("# {}", package.name)
}
