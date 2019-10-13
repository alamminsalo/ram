use super::assets::Assets;
use super::util;
use super::Field;
use failure::{format_err, Fallible};
use inflector::Inflector;
use mustache::{Data, MapBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Lang {
    pub name: String,
    pub types: HashMap<String, Type>,
    #[serde(default)]
    pub format: HashMap<String, String>,
    #[serde(default)]
    pub additional_files: Vec<AddFile>,
    pub paths: HashMap<String, String>,
    pub templates: HashMap<String, String>,
    #[serde(default)]
    pub reserved: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddFile {
    pub filename: String,
    pub template: String,
    pub r#in: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Type {
    #[serde(default)]
    pub alias: Vec<String>,
    pub format: HashMap<String, Format>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Format {
    pub r#type: String,
}

// returns common variations map from given field name + value
fn common_variables(value: &str) -> Data {
    MapBuilder::new()
        .insert_str("value", value)
        .insert_str("value_lowercase", value.to_lowercase())
        .insert_str("value_uppercase", value.to_uppercase())
        .insert_str("value_pascalcase", value.to_pascal_case())
        .insert_str("value_snakecase", value.to_snake_case())
        .insert_str("value_screamingsnakecase", value.to_screaming_snake_case())
        .build()
}

impl Lang {
    pub fn load_file(path: &str) -> Fallible<Self> {
        let data = {
            let mut path = path.to_string();
            // naive check if lang spec is not a file path
            // if not, assume it's one of the built-in lang specs
            if !path.contains("/") {
                // load from assets
                path = format!("{lang}/{lang}.yaml", lang = &path);
            }

            Assets::read_file(&path)?
        };

        let ext = Path::new(path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("yaml");

        let lang: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_str(&data)?,
            "json" | _ => serde_json::from_str(&data)?,
        };

        Ok(lang)
    }

    pub fn format(&self, template_key: &str, value: &str) -> Fallible<String> {
        let v = value.to_owned();
        if template_key == "reserved" && !self.reserved.contains(&v) {
            Ok(v)
        } else {
            self.format
                .get(template_key)
                .and_then(|t| mustache::compile_str(&t).ok())
                .and_then(|t| t.render_data_to_string(&common_variables(value)).ok())
                .ok_or(format_err!("failed to format template {}", template_key))
        }
    }

    pub fn translate(&self, f: Field) -> Field {
        let mut translated_type: String = if let Some(ref refpath) = f.ref_path {
            // this is a reference to another object
            let t = util::model_name_from_ref(&refpath).expect("failed to get model name from ref");
            // object field is not mandatory formatter rule
            self.format("object_field", &t).unwrap_or(t)
        } else {
            // this is a primitive language type
            let primitive_type = self
                .types
                .iter()
                .find(|(name, t)| *name == &f.r#type || t.alias.contains(&f.r#type))
                .map(|(_, t)| t)
                .expect(&format!("failed to find primitive type: {}", f.r#type));
            let type_format = f.format.clone().unwrap_or("default".into());
            primitive_type
                .format
                .get(&type_format)
                .expect(&format!("failed to find primitive type: {}", &f.r#type))
                .r#type
                .clone()
        };

        // format name if needed
        let name = self.format("reserved", &f.name).unwrap_or(f.name);

        if f.nullable {
            translated_type = self
                .format("nullable", &translated_type)
                .unwrap_or(translated_type)
        };

        Field {
            r#type: translated_type,
            name,
            ..f
        }
    }

    pub fn default_path(&self, path: &str) -> String {
        self.paths
            .get(path)
            .expect(&format!("failed to find default path: {}", path))
            .clone()
    }

    pub fn default_template(&self, path: &str) -> String {
        self.templates
            .get(path)
            .expect(&format!("failed to find default template: {}", path))
            .clone()
    }
}
