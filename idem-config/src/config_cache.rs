use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::fs;

static FILE_CACHE: Lazy<DashMap<String, Arc<String>>> = Lazy::new(DashMap::new);

pub fn get_file(file_path: &str) -> Result<Arc<String>, String> {
    if let Some(contents) = FILE_CACHE.get(file_path) {
        return Ok(contents.clone());
    }
    let contents = Arc::new(
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?,
    );
    FILE_CACHE.insert(file_path.to_string(), contents.clone());
    Ok(contents)
}

pub fn init_or_replace_config(file_path: &str) -> Result<(), String> {
    let contents = Arc::new(
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?,
    );
    FILE_CACHE.insert(file_path.to_string(), contents);
    Ok(())
}

pub fn clear_cache() {
    FILE_CACHE.clear();
}

#[cfg(test)]
mod test {
    use crate::config_cache::{clear_cache, get_file};
    use std::sync::Arc;

    #[test]
    fn test_cache() {
        let file_arc1 = get_file("./test/test.file").unwrap();
        let file_arc2 = get_file("./test/test.file").unwrap();
        assert!(Arc::ptr_eq(&file_arc1, &file_arc2));

        clear_cache();
        let file_arc3 = get_file("./test/test.file").unwrap();
        assert!(!Arc::ptr_eq(&file_arc1, &file_arc3));
    }
}
