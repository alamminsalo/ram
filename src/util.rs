use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// writes files in map
pub fn write_files(path: &Path, map: HashMap<String, String>) {
    // make sure directory exists
    fs::create_dir_all(path).expect("failed to create directory");
    map.iter().for_each(|(file, data)| {
        fs::write(path.join(&file), data).expect(&format!("failed to write file {}", &file));
    });
}

pub fn write_files_nopath(map: HashMap<String, String>) {
    map.iter().for_each(|(file, data)| {
        // create directory from file path
        let mut pathbuf = PathBuf::from(file);
        pathbuf.pop();
        fs::create_dir_all(pathbuf).expect("failed to create directory");
        // write file
        fs::write(file, data).expect(&format!("failed to write file {}", &file));
    });
}

// Returns model name from ref path
pub fn model_name_from_ref(ref_path: &str) -> Option<String> {
    if let Some(idx) = ref_path.rfind('/') {
        Some(ref_path[idx + 1..].to_string())
    } else {
        None
    }
}
