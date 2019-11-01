use super::assets::Assets;
use super::util;
use super::Model;
use failure::{format_err, Fallible};
use handlebars::*;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
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

    /*
     * Formatter functions
     */
    pub fn format(&self, template_key: &str, value: &String) -> Fallible<String> {
        match template_key {
            "reserved" if !self.reserved.contains(&value) => Ok(value.clone()),
            _ => {
                let mut map = HashMap::new();
                map.insert("value", value.as_str());
                self.format_map(template_key, &map)
            }
        }
    }

    pub fn format_map(&self, template_key: &str, map: &HashMap<&str, &str>) -> Fallible<String> {
        let mut hb = util::handlebars();
        self.add_helpers(&mut hb);
        self.format
            .get(template_key)
            .and_then(|template| hb.render_template(template, map).ok())
            .ok_or(format_err!("failed to format template {}", template_key))
    }

    /*
     * Model translation functions
     */
    /// Translates model
    pub fn translate(&self, m: Model) -> Model {
        // TODO: enum & match
        let mut translated_type: String = if m.is_array {
            self.translate_array(&m)
        } else if m.is_object {
            if let Some(ref refpath) = m.ref_path {
                // this is a reference to another object
                // get model name from ref_path
                util::model_name_from_ref(&refpath)
                    .map(|t| self.translate_modelname(&t))
                    .expect("failed to get model name from ref")
            } else {
                // this is an inline object, which we name by it's key
                self.translate_modelname(&m.name)
            }
        } else {
            // this is a primitive language type
            self.translate_primitive(
                &m.r#type,
                m.format.as_ref().unwrap_or(&String::from("default")),
            )
        };

        // format if nullable
        if m.nullable {
            translated_type = self
                .format("nullable", &translated_type)
                .unwrap_or(translated_type)
        };

        Model {
            r#type: translated_type,
            fields: m
                .fields
                .into_iter()
                .map(|m| Box::new(self.translate(*m)))
                .collect(),
            additional_fields: m
                .additional_fields
                .and_then(|m| Some(Box::new(self.translate(*m)))),
            ..m
        }
    }

    // applies `classname` and `object_field` to input str
    fn translate_modelname(&self, name: &String) -> String {
        // format using `classname` formatter if present
        let modelname = self.format("classname", &name).unwrap_or(name.clone());
        // format using `object_field` formatter if present
        self.format("object_field", &modelname).unwrap_or(modelname)
    }

    // translates to array type by child item
    fn translate_array(&self, m: &Model) -> String {
        // translate child
        let child = self.translate(*m.items.as_ref().expect("array child type is None").clone());
        // array formatter
        self.format_map(
            "array",
            &hashmap!["type" => child.r#type.as_str(), "name" => m.name.as_str()],
        )
        .expect("no array formatter defined!")
    }

    // returns translated primitive type
    fn translate_primitive(&self, _type: &String, format: &String) -> String {
        self.types
            .iter()
            .find(|(name, t)| *name == _type || t.alias.contains(_type))
            .and_then(|(_, t)| t.format.get(format))
            .map(|f| f.r#type.clone())
            .expect(&format!(
                "Error while processing {}: failed to find primitive type {}",
                _type, format
            ))
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
                let param = h
                    .param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("")
                    .to_string();
                out.write(&lang.format("reserved", &param).unwrap_or(param))?;
                Ok(())
            };
            hb.register_helper("r", Box::new(reserved));
        }
        {
            let ext = move |h: &Helper,
                            _: &Handlebars,
                            c: &Context,
                            r: &mut RenderContext,
                            out: &mut dyn Output|
                  -> HelperResult {
                // get parameter from helper or throw an error
                let param = h
                    .param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("")
                    .to_string();
                // get value {param} from local context extensions
                let value: String = c
                    .navigate(".", &VecDeque::new(), &r.get_path(), &VecDeque::new())
                    .map(|local| {
                        local
                            .as_json()
                            .get("extensions")
                            .and_then(|ext| ext.as_object())
                            .and_then(|ext| ext.get(&param))
                            .and_then(|val| val.as_str())
                            .unwrap_or("")
                            .to_owned()
                    })
                    .unwrap_or_default();
                // write out value
                out.write(&value)?;
                Ok(())
            };
            hb.register_helper("ext", Box::new(ext));
        }
    }
}
