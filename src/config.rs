use super::Lang;
use failure::Fallible;
use openapi::OpenApi;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub openapi_spec: Option<String>,
    pub lang_spec: Option<String>,
    pub template: TemplateConfig,
}

#[derive(Debug, Deserialize)]
pub struct TemplateConfig {
    pub model: Option<String>,
    pub api: Option<String>,
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
        openapi::from_path(&self.openapi_spec.clone().expect("no openapi spec defined"))
            .map_err(|e| e.into())
    }

    pub fn get_lang(&self) -> Fallible<Lang> {
        let mut lang_spec = self.lang_spec.clone().expect("no lang spec defined");

        // naive check if lang spec is not a file path
        // if not, assume it's one of the built-in lang specs
        if !lang_spec.contains(".") {
            lang_spec = format!("lang/{}.yaml", &lang_spec);
        }

        Lang::load_file(&lang_spec)
    }
}
