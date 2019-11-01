use super::util;
use openapi::v3_0::{ObjectOrReference, Schema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Model {
    pub name: String,
    pub r#type: String,
    pub fields: Vec<Box<Model>>,
    pub additional_fields: Option<Box<Model>>,
    pub items: Option<Box<Model>>,
    pub has_date: bool,
    pub has_datetime: bool,
    pub is_array: bool,
    pub is_object: bool,
    pub description: Option<String>,
    pub format: Option<String>,
    pub nullable: bool,
    pub ref_path: Option<String>,
    pub extensions: HashMap<String, String>,
}

impl Model {
    pub fn new(name: &str, schema: &Schema) -> Self {
        let fields: Vec<Box<Model>> = schema
            .properties
            .iter()
            .flatten()
            .map(|(name, schema)| Box::new(Model::new(&name, schema)))
            .collect();

        let additional_fields: Option<Box<Model>> =
            schema
                .additional_properties
                .as_ref()
                .and_then(|obj_or_ref| match obj_or_ref {
                    ObjectOrReference::Object(s) => Some(Box::new(Model::new("", &s))),
                    _ => None,
                });

        let t = schema
            .schema_type
            .as_ref()
            .unwrap_or(&String::from("object"))
            .to_owned();
        let is_array = &t == "array";
        let is_object = &t == "object";

        Self {
            // TODO: sense
            name: name.into(),
            // checks if any field contains format: date
            has_date: fields.iter().any(|f| {
                if let Some(fieldformat) = &f.format {
                    fieldformat == "date"
                } else {
                    false
                }
            }),
            ref_path: schema.ref_path.clone(),
            // checks if any field contains format: date-time
            has_datetime: fields.iter().any(|f| {
                if let Some(fieldformat) = &f.format {
                    fieldformat == "date-time"
                } else {
                    false
                }
            }),
            items: schema.items.as_ref().map(|s| {
                let name = s
                    .ref_path
                    .as_ref()
                    .map(|ref_path| {
                        util::model_name_from_ref(&ref_path)
                            .expect("failed to get model name from ref_path")
                    })
                    .unwrap_or(String::new());
                Box::new(Model::new(&name, &s))
            }),
            nullable: schema.nullable.unwrap_or(false),
            description: schema.description.clone(),
            format: schema.format.clone(),
            extensions: schema.extensions.clone(),
            r#type: t,
            fields,
            additional_fields,
            is_array,
            is_object,
        }
    }
}
