use super::util;
use super::{AddFile, GroupingStrategy, Lang};
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

    // Defines language or direct path to custom lang spec
    pub lang: String,

    #[serde(default)]
    pub paths: HashMap<String, String>,

    // custom formatters, these are added to lang formatters
    #[serde(default)]
    pub helpers: HashMap<String, String>,

    /// Additional files to generate
    #[serde(default)]
    pub files: Vec<AddFile>,

    #[serde(default)]
    pub grouping_strategy: Option<GroupingStrategy>,
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
        let f = &self.lang;

        // if file has extension set, assume its a path to file and join path
        let mut path = PathBuf::from(f);
        if path.extension().is_some() {
            path = util::join_relative(&self.path, &path);
        }
        // load lang file
        Lang::load_file(&path).and_then(|mut lang| {
            // add custom formatters to lang formatters
            lang.helpers.extend(self.helpers.clone());
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

    pub fn get_files(&self, lang: Option<&Lang>) -> Vec<AddFile> {
        let config_files = self.files.iter().map(|f: &AddFile| {
            // join relative cfg path
            let template = util::join_relative(&self.path, &PathBuf::from(&f.template))
                .to_str()
                .unwrap()
                .to_owned();
            AddFile {
                template,
                ..f.clone()
            }
        });

        lang.into_iter()
            .flat_map(|l| l.files_relative())
            .chain(config_files)
            .collect()
    }
}
