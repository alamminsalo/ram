use super::assets::Assets;
use super::util;
use super::Model;
use failure::{format_err, Fallible};
use handlebars::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddFile {
    pub filename: String,
    pub template: String,
    pub r#in: Option<String>,
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
    pub r#type: String,
}

// Value type for formatting
#[derive(Serialize, Deserialize)]
struct Value {
    pub value: String,
}

impl From<&str> for Value {
    fn from(a: &str) -> Value {
        Value { value: a.into() }
    }
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

    // fn is_primitive(&self, t: &str) -> bool {
    // }

    pub fn format(&self, template_key: &str, value: &str) -> Fallible<String> {
        let mut hb = util::handlebars();
        self.add_helpers(&mut hb);
        let v = value.to_owned();
        match template_key {
            "reserved" if !self.reserved.contains(&v) => Ok(v),
            // "classname" if self.is_primitive(&v) => Ok(v),
            _ => self
                .format
                .get(template_key)
                .and_then(|template| hb.render_template(template, &Value::from(value)).ok())
                .ok_or(format_err!("failed to format template {}", template_key)),
        }
    }

    pub fn translate(&self, f: Model) -> Model {
        let mut translated_type: String = if f.is_array {
            let child_type = f
                .items
                .as_ref()
                .map(|s| {
                    s.ref_path
                        .as_ref()
                        .map(|ref_path| {
                            self.format(
                                "classname",
                                &util::model_name_from_ref(&ref_path)
                                    .expect("failed to get model name from ref_path"),
                            )
                            .expect("failed to format classname")
                        })
                        .unwrap_or(s.r#type.clone())
                })
                .expect("array child type not defined!");
            // format as array<child type>
            self.format("array_field", &child_type)
                .expect("no array formatter defined!")
        } else if let Some(ref refpath) = f.ref_path {
            // this is a reference to another object
            let t = util::model_name_from_ref(&refpath)
                .map(|t| {
                    self.format("classname", &t)
                        .expect("classname formatting failed")
                })
                .expect("failed to get model name from ref");
            // object field is not mandatory formatter rule
            self.format("object_field", &t).unwrap_or(t)
        } else if f.is_object {
            // this is an inline object, which is not yet supported
            panic!("{}: inline objects are not supported", f.name);
        } else {
            // this is a primitive language type
            let primitive_type = self
                .types
                .iter()
                .find(|(name, t)| *name == &f.r#type || t.alias.contains(&f.r#type))
                .map(|(_, t)| t)
                .expect(&format!(
                    "Error while processing {}: failed to find primitive type {}",
                    &f.name, f.r#type
                ));
            let type_format = f.format.clone().unwrap_or("default".into());
            primitive_type
                .format
                .get(&type_format)
                .expect(&format!(
                    "Error while processing {}: failed to find primitive type {}",
                    &f.name, &f.r#type
                ))
                .r#type
                .clone()
        };

        if f.nullable {
            translated_type = self
                .format("nullable", &translated_type)
                .unwrap_or(translated_type)
        };

        Model {
            r#type: translated_type,
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

    // adds helpers to handlebars instance
    pub fn add_helpers(&self, hb: &mut Handlebars) {
        {
            let lang = self.clone();
            let reserved = move |h: &Helper,
                                 _: &Handlebars,
                                 _: &Context,
                                 _: &mut RenderContext,
                                 out: &mut dyn Output|
                  -> HelperResult {
                // get parameter from helper or throw an error
                let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
                out.write(&lang.format("reserved", &param).unwrap_or(param.to_string()))?;
                Ok(())
            };
            hb.register_helper("r", Box::new(reserved));
        }
    }
}
