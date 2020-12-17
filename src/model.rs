use super::lang::Lang;
use super::util;
use indexmap::IndexMap;
use openapi::v3_0::{ObjectOrReference, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ModelType {
    Primitive,
    Object,
    Array,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    pub def: String,
    pub name: String,
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Vec<Box<Model>>,
    pub readonly: bool,
    pub additional_properties: Option<Box<Model>>,
    pub items: Option<Box<Model>>,
    pub description: Option<String>,
    pub format: Option<String>,
    pub nullable: bool,
    #[serde(skip)]
    pub ref_path: Option<String>,

    /// Model extensions.
    /// Used for additional non-openapi specific information.
    /// Examples: `x-sql-table`, `x-go-tag`
    /// Flattened: use directly from model `{{ x-sql-name }}`
    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,

    // additional helper properties, these are derived from the 'base' properties
    pub is_object: bool,
    pub is_array: bool,
    pub is_primitive: bool,
    pub has_date: bool,
    pub has_datetime: bool,
    pub object_properties: Vec<Box<Model>>,
    pub array_properties: Vec<Box<Model>>,
    pub primitive_properties: Vec<Box<Model>>,
}

impl Model {
    pub fn new(name: &str, schema: &Schema, def: &str) -> Self {
        let properties: Vec<Box<Model>> = schema
            .properties
            .iter()
            .flatten()
            .map(|(name, schema)| Box::new(Model::new(&name, schema, "")))
            .collect();

        let additional_properties: Option<Box<Model>> = schema
            .additional_properties
            .as_ref()
            .and_then(|obj_or_ref| match obj_or_ref {
                ObjectOrReference::Object(s) => Some(Box::new(Model::new("", &s, ""))),
                _ => None,
            });

        let schema_type = schema
            .schema_type
            .as_ref()
            .unwrap_or(&String::from("object"))
            .to_owned();

        // If input name is "", try to extract one from ref_path.
        // Otherwise use the name.
        let def: String = if def == "" {
            util::extract_model_name(schema).unwrap_or_default()
        } else {
            def.into()
        };

        let mut model = Model {
            name: name.into(),
            ref_path: schema.ref_path.clone(),
            items: schema.items.as_ref().map(|s| {
                let name = util::extract_model_name(s).unwrap_or_default();
                Box::new(Model::new(&name, &s, ""))
            }),
            nullable: schema.nullable.unwrap_or(false),
            description: schema.description.clone(),
            format: schema.format.clone(),
            extensions: schema.extensions.clone(),
            readonly: schema.read_only.unwrap_or(false),
            def,
            schema_type,
            properties,
            additional_properties,
            ..Default::default()
        };

        // do this only once in creation
        model.apply_properties();

        // add helpers
        model
    }

    fn apply_properties(&mut self) {
        self.set_has_date();
        self.set_has_datetime();
        self.set_is_object();
        self.set_is_array();
        self.set_is_primitive();

        // set child properties
        for child in self.properties.iter_mut() {
            child.apply_properties();
        }

        self.set_object_properties();
        self.set_array_properties();
        self.set_primitive_properties();
    }

    // checks if any field contains format: date
    fn set_has_date(&mut self) {
        self.has_date = self.properties.iter().any(|f| {
            if let Some(fieldformat) = &f.format {
                fieldformat == "date"
            } else {
                false
            }
        })
    }

    // checks if any field contains format: datetime
    fn set_has_datetime(&mut self) {
        self.has_datetime = self.properties.iter().any(|f| {
            if let Some(fieldformat) = &f.format {
                fieldformat == "date-time"
            } else {
                false
            }
        })
    }

    fn set_is_object(&mut self) {
        self.is_object = self.schema_type == "object"
    }

    fn set_is_array(&mut self) {
        self.is_array = self.schema_type == "array"
    }

    fn set_is_primitive(&mut self) {
        self.is_primitive = !self.is_array && !self.is_object
    }

    fn set_primitive_properties<'a>(&'a mut self) {
        self.primitive_properties = self
            .properties
            .iter()
            .cloned()
            .chain(
                self.additional_properties
                    .iter()
                    .cloned()
                    .flat_map(|p| p.primitive_properties),
            )
            .filter(|f| f.is_primitive)
            .collect();
    }

    fn set_object_properties<'a>(&'a mut self) {
        self.object_properties = self
            .properties
            .iter()
            .cloned()
            .chain(
                self.additional_properties
                    .iter()
                    .cloned()
                    .flat_map(|p| p.object_properties),
            )
            .filter(|f| f.is_object)
            .collect()
    }

    fn set_array_properties<'a>(&'a mut self) {
        self.array_properties = self
            .properties
            .iter()
            .cloned()
            .chain(
                self.additional_properties
                    .iter()
                    .cloned()
                    .flat_map(|p| p.array_properties),
            )
            .filter(|f| f.is_array)
            .collect()
    }

    pub fn model_type(&self) -> ModelType {
        if self.is_array {
            ModelType::Array
        } else if self.is_object {
            ModelType::Object
        } else {
            ModelType::Primitive
        }
    }

    // translates model
    pub fn translate(self, lang: &Lang) -> Model {
        let mut translated_type = match self.model_type() {
            ModelType::Array => lang.translate_array(&self),
            ModelType::Object => {
                if let Some(ref refpath) = self.ref_path {
                    // this is a reference to another object
                    // get model name from ref_path
                    util::model_name_from_ref(&refpath)
                        .map(|t| lang.translate_modelname(&t))
                        .expect("failed to get model name from ref")
                } else {
                    // this is an inline object, which we name by it's key
                    lang.translate_modelname(&self.name)
                }
            }
            ModelType::Primitive => lang.translate_primitive(
                &self.schema_type,
                self.format.as_ref().unwrap_or(&String::from("default")),
            ),
        };

        // format if nullable
        if self.nullable {
            translated_type = lang
                .format("nullable", &translated_type)
                .unwrap_or(translated_type)
        };

        Model {
            schema_type: translated_type,
            properties: self
                .properties
                .into_iter()
                .map(|m| Box::new(m.translate(lang)))
                .collect(),
            additional_properties: self
                .additional_properties
                .and_then(|m| Some(Box::new(m.translate(lang)))),
            primitive_properties: self
                .primitive_properties
                .into_iter()
                .map(|m| Box::new(m.translate(lang)))
                .collect(),
            object_properties: self
                .object_properties
                .into_iter()
                .map(|m| Box::new(m.translate(lang)))
                .collect(),
            array_properties: self
                .array_properties
                .into_iter()
                .map(|m| Box::new(m.translate(lang)))
                .collect(),
            ..self
        }
    }

    // normalizes child refs (clones object from input map)
    pub fn normalize(self, models_map: &HashMap<String, Self>) -> Self {
        Self {
            object_properties: self
                .object_properties
                .into_iter()
                .map(|m| {
                    models_map
                        .get(&m.def)
                        .expect(&format!("failed to get model '{}' from map", &m.def))
                })
                .cloned()
                .map(Box::new)
                .collect(),

            array_properties: self
                .array_properties
                .into_iter()
                .map(|m| {
                    let mut items: Box<Model> = m.items.expect("array item was None").clone();
                    if items.is_object {
                        items = Box::new(
                            models_map
                                .get(&items.def)
                                .expect(&format!(
                                    "failed to get array item '{}' from map",
                                    &items.def
                                ))
                                .clone(),
                        )
                    }
                    Model {
                        items: Some(items),
                        ..*m
                    }
                })
                .map(Box::new)
                .collect(),

            ..self
        }
    }
}
