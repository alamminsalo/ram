use super::helper;
use glob::Pattern;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// returns ignore patterns from '.ramignore'
fn ignore_patterns() -> Vec<Pattern> {
    fs::read_to_string(".ramignore")
        .and_then(|contents| {
            Ok(contents
                .split("\n")
                .filter_map(|line| Pattern::new(line).ok())
                .collect())
        })
        .unwrap_or(vec![])
}

// writes files in map
pub fn write_files(path: &Path, map: HashMap<String, String>) {
    let ignored = ignore_patterns();
    // make sure directory exists
    fs::create_dir_all(path).expect("failed to create directory");
    for (file, data) in map.iter() {
        let path = path.join(&file);
        if ignored.iter().any(|p| p.matches_path(&path)) {
            println!("ignoring file {}", path.to_str().unwrap_or(""));
            continue;
        }
        fs::write(path, data).expect(&format!("failed to write file {}", &file));
    }
}

pub fn write_files_nopath(map: HashMap<String, String>) {
    let ignored = ignore_patterns();
    for (file, data) in map.iter() {
        let mut path = PathBuf::from(file);
        if ignored.iter().any(|p| p.matches_path(&path)) {
            println!("ignoring file {}", path.to_str().unwrap_or(""));
            continue;
        }
        // create directory from file path
        path.pop();
        fs::create_dir_all(path).expect("failed to create directory");
        // write file
        fs::write(file, data).expect(&format!("failed to write file {}", &file));
    }
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

    // register custom helpers
    helper::register_helpers(&mut hb);

    hb
}
