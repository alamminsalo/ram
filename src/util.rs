use std::collections::HashMap;
use std::fs::{self, DirBuilder};
use std::path::Path;

// writes files in map
pub fn write_files(path: &Path, map: HashMap<String, String>) {
    // make sure directory exists
    DirBuilder::new()
        .recursive(true)
        .create(path)
        .expect("failed to create directory");
    map.iter().for_each(|(file, data)| {
        println!("{}: {}", &file, &data);
        fs::write(path.join(&file), data).expect(&format!("failed to write file {}", &file));
    });
}
