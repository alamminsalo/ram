use std::collections::HashMap;
use std::fs;

// writes files in map
pub fn write_files(map: HashMap<String, String>) {
    map.iter().for_each(|(file, data)| {
        println!("{}: {}", &file, &data);
        fs::write(&file, data).expect(&format!("failed to write file {}", &file));
    });
}
