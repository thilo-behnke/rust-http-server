pub mod path {
    use std::path::{Path, PathBuf, MAIN_SEPARATOR};

    pub fn remap(path: &Path, base_path: &Path, mapping_base: &Path) -> PathBuf {
        if !path.starts_with(base_path) {
            return PathBuf::from(path);
        }
        Path::new(&MAIN_SEPARATOR.to_string())
            .join(mapping_base.join(path.strip_prefix(base_path).unwrap()))
    }

    pub fn join_mapped(base_path: &Path, path: &Path) -> PathBuf {
        let mapped_path = match path.starts_with(&MAIN_SEPARATOR.to_string()) {
            true => {
                let path: PathBuf = path.to_path_buf().iter().skip(1).collect::<PathBuf>();
                path
            }
            false => path.to_path_buf(),
        };
        return base_path.join(mapped_path);
    }
}
