use super::{Lang, Model};
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
    pub template: HashMap<String, String>,
    pub namespace: HashMap<String, String>,
    pub paths: HashMap<String, String>,
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
        let mut lang = self.lang.clone().expect("no lang spec defined");

        // naive check if lang spec is not a file path
        // if not, assume it's one of the built-in lang specs
        if !lang.contains(".") {
            lang = format!("lang/{lang}/{lang}.yaml", lang = &lang);
        }

        Lang::load_file(&lang)
    }

    // returns model path using lang specs filename property
    pub fn model_path(&self, model: &Model, lang: &Lang) -> String {
        lang.format_filename(
            mustache::MapBuilder::new()
                .insert_str("filename", model.name.to_lowercase())
                .build(),
        )
    }
}
