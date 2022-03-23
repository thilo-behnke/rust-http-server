pub mod path {
    use std::path::{Path, PathBuf, MAIN_SEPARATOR};

    pub fn remap(path: &Path, base_path: &Path, mapping_base: &Path) -> PathBuf {
        if !path.starts_with(base_path) {
            return PathBuf::from(path);
        }
        Path::new(&MAIN_SEPARATOR.to_string())
            .join(mapping_base.join(path.strip_prefix(base_path).unwrap()))
    }
}
