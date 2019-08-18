use failure::{format_err, Error, Fallible};
use openapi::OpenApi;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub spec: String,
    pub lang: String,
    pub models: String,
    pub api: String,
}

impl Config {
    pub fn load_file(path: &str) -> Fallible<Self> {
        let path = Path::new(path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ext = path.extension().expect("failed to get extension");
        let ext: &str = ext.to_str().expect("failed to read extension");

        let cfg: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_reader(reader)?,
            "json" | _ => serde_json::from_reader(reader)?,
        };

        Ok(cfg)
    }

    pub fn openapi_spec(&self) -> Fallible<OpenApi> {
        openapi::from_path(&self.spec).map_err(|e| e.into())
    }
}

// language-specific types
pub struct Lang {
    map: HashMap<String, String>,
}

impl Lang {
    pub fn load(&self, path: &str) -> Fallible<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let map = serde_json::from_reader(reader)?;
        Ok(Self { map })
    }

    // Returns value or key if not found
    pub fn t(&self, k: &str) -> String {
        self.map.get(k).unwrap_or(&k.into()).clone()
    }
}
