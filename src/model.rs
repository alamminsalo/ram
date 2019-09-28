use super::lang::Lang;
use super::util;
use openapi::v3_0::{ObjectOrReference, Schema};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct Model {
    pub name: String,
    pub name_lowercase: String,
    pub r#type: String,
    pub fields: Vec<Field>,
    pub additional_fields: Option<Box<Model>>,
    pub items: Option<Box<Model>>,
    pub has_date: bool,
    pub has_datetime: bool,
    pub is_array: bool,
    pub is_object: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    pub format: Option<String>,
    pub nullable: bool,
    pub ref_path: Option<String>,
    pub is_array: bool,
}

impl Model {
    pub fn new(name: &str, schema: Schema, lang: &Lang) -> Self {
        let fields: Vec<Field> = schema
            .properties
            .unwrap_or(BTreeMap::new())
            .into_iter()
            .map(|(name, schema)| {
                // translate using language spec
                lang.translate(Field {
                    nullable: schema.nullable.unwrap_or(false),
                    format: schema.format,
                    ref_path: schema.ref_path,
                    is_array: schema
                        .schema_type
                        .clone()
                        .into_iter()
                        .any(|t| &t == "array"),
                    r#type: schema.schema_type.unwrap_or("object".into()),
                    name,
                })
            })
            .collect();

        let additional_fields: Option<Box<Model>> =
            schema
                .additional_properties
                .and_then(|obj_or_ref| match obj_or_ref {
                    ObjectOrReference::Object(s) => Some(Box::new(Model::new("", *s, &lang))),
                    _ => None,
                });

        let r#type = schema.schema_type.unwrap_or("object".into());
        let is_array = &r#type == "array";
        let is_object = &r#type == "object";

        Self {
            name: name.to_string(),
            name_lowercase: name.to_lowercase(),
            // checks if any field contains format: date
            has_date: fields.iter().any(|f| {
                if let Some(fieldformat) = &f.format {
                    fieldformat == "date"
                } else {
                    false
                }
            }),
            // checks if any field contains format: date-time
            has_datetime: fields.iter().any(|f| {
                if let Some(fieldformat) = &f.format {
                    fieldformat == "date-time"
                } else {
                    false
                }
            }),
            items: schema.items.map(|s| {
                Box::new(Model::new(
                    &util::model_name_from_ref(&s.ref_path.clone().unwrap()).unwrap(),
                    *s,
                    lang,
                ))
            }),
            fields,
            additional_fields,
            r#type,
            is_array,
            is_object,
        }
    }
}
