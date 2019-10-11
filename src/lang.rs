use super::util;
use super::Field;
use failure::Fallible;
use inflector::Inflector;
use mustache::{Data, MapBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Lang {
    pub name: String,
    pub types: HashMap<String, Type>,
    #[serde(default)]
    pub format: HashMap<String, String>,
    pub files: Vec<ExtraFile>,
    pub paths: HashMap<String, String>,
    pub templates: HashMap<String, String>,
    pub reserved: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtraFile {
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
        let path = Path::new(path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ext = path.extension().expect("failed to get extension");
        let ext: &str = ext.to_str().expect("failed to read extension");

        let lang: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_reader(reader)?,
            "json" | _ => serde_json::from_reader(reader)?,
        };

        Ok(lang)
    }

    pub fn format(&self, template_key: &str, value: &str) -> String {
        let t = self
            .format
            .get(template_key)
            .clone()
            .expect("no formatting template found");
        let template = mustache::compile_str(&t).expect("failed to compile filename template");
        template
            .render_data_to_string(&common_variables(value))
            .expect("failed to format filename")
    }

    pub fn translate(&self, f: Field) -> Field {
        let mut translated_type: String = if let Some(ref refpath) = f.ref_path {
            // this is a reference to another object
            util::model_name_from_ref(&refpath).expect("failed to get model name from ref")
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
        let name = if self.reserved.contains(&f.name) {
            self.format("reserved", &f.name)
        } else {
            f.name.clone()
        };

        if f.nullable {
            translated_type = self.format("nullable", &translated_type);
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
