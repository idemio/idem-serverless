use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
static FILE_CACHE: LazyLock<RwLock<HashMap<String, Arc<String>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn get_config_file(file_path: &str) -> Result<Arc<String>, ()> {
    {
        let cache = FILE_CACHE.read().unwrap();
        if let Some(contents) = cache.get(file_path) {
            return Ok(contents.clone());
        }
    }

    let contents = Arc::new(
        std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))
            .unwrap(),
    );

    {
        let mut cache = FILE_CACHE.write().unwrap();
        cache.insert(file_path.to_string(), contents.clone());
    }

    Ok(contents)
}

pub fn init_or_replace_config(file_path: &str) -> Result<(), ()> {
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            let content = Arc::new(content);
            FILE_CACHE
                .write()
                .unwrap()
                .insert(file_path.to_string(), content.clone());
            Ok(())
        }
        Err(_) => Err(()),
    }
}

pub fn clear_cache() {
    FILE_CACHE.write().unwrap().clear();
}

#[cfg(test)]
mod test {
    use crate::config_cache::{clear_cache, get_config_file};
    use std::sync::Arc;

    #[test]
    fn test_cache() {
        let file_arc1 = get_config_file("./test/test.file").unwrap();
        let file_arc2 = get_config_file("./test/test.file").unwrap();
        assert!(Arc::ptr_eq(&file_arc1, &file_arc2));

        clear_cache();
        let file_arc3 = get_config_file("./test/test.file").unwrap();
        assert!(!Arc::ptr_eq(&file_arc1, &file_arc3));
    }
}
