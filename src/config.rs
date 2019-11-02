use super::{AddFile, Lang};
use failure::Fallible;
use openapi::OpenApi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub openapi: Option<String>,

    pub lang: Option<String>,

    #[serde(default)]
    pub templates: HashMap<String, String>,

    #[serde(default)]
    pub paths: HashMap<String, String>,

    /// Additional files to generate
    pub additional_files: Vec<AddFile>,
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

    pub fn get_openapi(&self) -> Fallible<OpenApi> {
        openapi::from_path(&self.openapi.clone().expect("no openapi spec defined"))
            .map_err(|e| e.into())
    }

    pub fn get_lang(&self) -> Fallible<Lang> {
        let lang = self.lang.as_ref().expect("no lang spec defined");
        Lang::load_file(&lang)
    }

    pub fn get_rootpath(&self, lang: &Lang) -> String {
        self.paths
            .get("root")
            .unwrap_or(&lang.default_path("root"))
            .clone()
    }

    // Returns formatted path according to config / lang spec defaults
    pub fn get_path(&self, path_key: &str, lang: &Lang) -> String {
        let root = self.get_rootpath(lang);
        let path: String = self
            .paths
            .get(path_key)
            .and_then(|p| Some(p.clone()))
            .or_else(|| Some(lang.default_path(path_key)))
            .unwrap();
        Path::new(&root).join(&path).to_str().unwrap().to_string()
    }

    // Returns template or language default
    pub fn get_template(&self, path_key: &str, lang: &Lang) -> String {
        self.templates
            .get(path_key)
            .and_then(|t| Some(t.clone()))
            .or_else(|| Some(lang.default_template(path_key)))
            .unwrap()
    }
}
