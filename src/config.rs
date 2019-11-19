use super::{AddFile, Lang};
use failure::Fallible;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,

    pub lang: Option<String>,

    #[serde(default)]
    pub templates: HashMap<String, String>,

    #[serde(default)]
    pub paths: HashMap<String, String>,

    // custom formatters, these are added to lang formatters
    #[serde(default)]
    pub format: HashMap<String, String>,

    /// Additional files to generate
    #[serde(default)]
    pub additional_files: Vec<AddFile>,
}

impl Config {
    pub fn load_file(path: &Path) -> Fallible<Config> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ext = path.extension().expect("failed to get extension");
        let ext: &str = ext.to_str().expect("failed to read extension");

        let mut cfg: Config = match ext {
            "yaml" | "yml" => serde_yaml::from_reader(reader)?,
            "json" | _ => serde_json::from_reader(reader)?,
        };

        // set cfg path
        cfg.path = path.canonicalize().unwrap().parent().unwrap().into();

        Ok(cfg)
    }

    pub fn get_lang(&self) -> Fallible<Lang> {
        let f = self.lang.as_ref().expect("no lang spec defined");
        // load lang file
        Lang::load_file(&PathBuf::from(f)).and_then(|mut lang| {
            // add custom formatters to lang formatters
            lang.format.extend(self.format.clone());
            Ok(lang)
        })
    }

    // Returns formatted path according to config / lang spec defaults
    pub fn get_path(&self, path_key: &str, lang: &Lang) -> PathBuf {
        self.paths
            .get(path_key)
            .and_then(|p| Some(PathBuf::from(&p)))
            .or_else(|| Some(lang.default_path(path_key)))
            .unwrap()
    }

    // Returns template or language default
    pub fn get_template(&self, path_key: &str, lang: &Lang) -> PathBuf {
        self.templates
            .get(path_key)
            .and_then(|t| Some(self.join_path(&PathBuf::from(&t))))
            .or_else(|| Some(lang.default_template(path_key)))
            .unwrap()
    }

    fn join_path(&self, p: &Path) -> PathBuf {
        if p.is_relative() {
            self.path.join(p)
        } else {
            PathBuf::from(p)
        }
    }

    pub fn get_additional_files(&self, lang: &Lang) -> Vec<AddFile> {
        lang.additional_files
            .iter()
            .cloned()
            .chain(self.additional_files.iter().map(|f: &AddFile| {
                // join relative cfg path
                let template = self.path.join(&f.template).to_str().unwrap().to_owned();
                AddFile {
                    template,
                    ..f.clone()
                }
            }))
            .collect()
    }
}
