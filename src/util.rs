use super::helper;
use handlebars::Handlebars;
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

pub fn handlebars() -> Handlebars {
    let mut hb = Handlebars::new();

    // set strict mode (fails on field not found)
    hb.set_strict_mode(true);

    // disable html escaping
    hb.register_escape_fn(handlebars::no_escape);

    hb.register_helper("lowercase", Box::new(helper::lowercase));
    hb.register_helper("uppercase", Box::new(helper::uppercase));
    hb.register_helper("pascalcase", Box::new(helper::pascalcase));
    hb.register_helper("snakecase", Box::new(helper::snakecase));
    hb.register_helper("screamingcase", Box::new(helper::screamingcase));

    hb
}
