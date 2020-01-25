use super::assets::Assets;
use super::util;
use super::Model;
use failure::Fallible;
use handlebars::Handlebars;
use handlebars::*;
use itertools::Itertools;
use maplit::hashmap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Lang {
    #[serde(skip)]
    pub path: PathBuf,

    pub name: String,
    #[serde(default)]
    pub types: HashMap<String, Type>,
    #[serde(default)]
    pub format: HashMap<String, String>,
    #[serde(default)]
    pub files: Vec<AddFile>,
    #[serde(default)]
    pub paths: HashMap<String, String>,
    #[serde(default)]
    pub templates: HashMap<String, String>,
    #[serde(default)]
    pub reserved: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddFile {
    pub filename: Option<String>,
    pub template: String,
    #[serde(rename = "in")]
    pub file_in: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Type {
    #[serde(default)]
    pub alias: Vec<String>,
    pub format: HashMap<String, Format>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Format {
    #[serde(rename = "type")]
    pub schema_type: String,
}

impl Lang {
    pub fn load_file(path: &Path) -> Fallible<Self> {
        let mut pathbuf = path.to_owned();
        let data = {
            // if no extension, assume its one of the built-in specs
            if path.extension().is_none() {
                // load from assets
                pathbuf = PathBuf::from(&format!(
                    "{lang}/{lang}.yaml",
                    lang = &path.to_str().unwrap()
                ));
            }

            Assets::read_file(&PathBuf::from(&pathbuf))?
        };

        let ext = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("yaml");

        let mut lang: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_str(&data)?,
            "json" | _ => serde_json::from_str(&data)?,
        };

        // set lang spec path
        lang.path = pathbuf
            .parent()
            .expect("failed to get lang parent dir")
            .to_owned();

        // set root path to "" if not set
        if lang.paths.get("root") == None {
            lang.paths.insert("root".into(), "".into());
        }

        Ok(lang)
    }

    pub fn default_path(&self, path: &str) -> PathBuf {
        PathBuf::from(
            &self
                .paths
                .get(path)
                .expect(&format!("failed to find default path: {}", path)),
        )
    }

    pub fn default_template(&self, path: &str) -> PathBuf {
        PathBuf::from(
            &self
                .templates
                .get(path)
                .map(|p| util::join_relative(&self.path, &PathBuf::from(&p)))
                .expect(&format!("failed to find default template: {}", path)),
        )
    }

    // Returns vec of additional files, with joined relative paths
    pub fn files_relative(&self) -> Vec<AddFile> {
        self.files
            .iter()
            .map(|af| AddFile {
                template: util::join_relative(&self.path, &PathBuf::from(&af.template))
                    .to_str()
                    .unwrap()
                    .into(),
                ..af.clone()
            })
            .collect()
    }

    /*
     * Formatter functions
     */
    pub fn format(&self, template_key: &str, value: &String) -> Fallible<String> {
        match template_key {
            "r" if !self.reserved.contains(&value) => Ok(value.clone()),
            _ => {
                let mut map = HashMap::new();
                map.insert("value", value.as_str());
                Ok(self.format_map(template_key, &map))
            }
        }
    }

    pub fn format_map(&self, template_key: &str, map: &HashMap<&str, &str>) -> String {
        let mut hb = Handlebars::new();
        util::init_handlebars(&mut hb);
        self.add_helpers(&mut hb);
        self.format
            .get(template_key)
            .and_then(|template| hb.render_template(template, map).ok())
            .unwrap_or_else(|| map.get("value").unwrap().to_string())
    }

    // applies `classname` and `object_property` to input str
    pub fn translate_modelname(&self, name: &String) -> String {
        // format using `classname` formatter if present
        let modelname = self.format("classname", &name).unwrap_or(name.clone());
        // format using `object_property` formatter if present
        self.format("object_property", &modelname)
            .unwrap_or(modelname)
    }

    // translates to array type by child item
    pub fn translate_array(&self, m: &Model) -> String {
        // translate child
        let child = m
            .items
            .as_ref()
            .expect("array child type is None")
            .clone()
            .translate(self);
        // array formatter
        self.format_map(
            "array",
            &hashmap!["value" => m.name.as_str(), "type" => child.schema_type.as_str(), "name" => m.name.as_str()],
        )
    }

    // returns translated primitive type
    pub fn translate_primitive(&self, schema_type: &String, format: &String) -> String {
        self.types
            .iter()
            .find(|(name, t)| *name == schema_type || t.alias.contains(schema_type))
            .and_then(|(_, t)| t.format.get(format).or_else(|| t.format.get("default")))
            .map(|f| f.schema_type.clone())
            .expect(&format!(
                "Error while processing {}: failed to find primitive type {}",
                schema_type, format
            ))
    }

    // adds helpers to handlebars instance
    pub fn add_helpers(&self, hb: &mut Handlebars) {
        // add custom formatter helpers
        for k in self.format.keys() {
            let lang = self.clone();
            let key = k.clone();
            let closure = move |h: &Helper,
                                _: &Handlebars,
                                _: &Context,
                                _: &mut RenderContext,
                                out: &mut dyn Output|
                  -> HelperResult {
                // get parameter from helper or throw an error
                let param = h
                    .param(0)
                    .and_then(|v| v.value().as_str())
                    .expect("parameter is missing")
                    .to_string();
                out.write(&lang.format(&key, &param).unwrap_or(param))?;
                Ok(())
            };
            hb.register_helper(k, Box::new(closure));
        }
    }

    /// Formats all path paramers in form of {param} with given formatter if any
    pub fn format_path(&self, p: String) -> String {
        // TODO: clean this mess
        let re = Regex::new(r"^\{(\w+)\}$").unwrap();
        self.format
            .get("pathparam")
            .map(|_| {
                format!(
                    "/{}",
                    &Path::new(&p)
                        .iter()
                        .skip(1) // leave out preceding '/', which is in the standard
                        .map(|part| part.to_str().unwrap())
                        .map(|part| {
                            if let Some(cap) = re.captures_iter(part).next() {
                                self.format("pathparam", &cap[1].to_owned())
                                    .unwrap_or(part.to_string())
                            } else {
                                part.to_string()
                            }
                        })
                        .join("/")
                )
            })
            .unwrap_or(p)
    }
}
