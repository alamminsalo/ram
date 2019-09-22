use super::lang::Lang;
use openapi::v3_0::Schema;
use serde::Serialize;
use std::collections::BTreeMap;

// Returns models name from ref path
fn name_from_ref(ref_path: &str) -> Option<String> {
    if let Some(idx) = ref_path.rfind('/') {
        Some(ref_path[idx + 1..].to_string())
    } else {
        None
    }
}

#[derive(Serialize, Debug)]
pub struct Model {
    pub model_name: String,
    pub r#type: String,
    pub fields: Vec<Field>,
    pub items: Option<Box<Model>>,
    pub has_date: bool,
    pub has_datetime: bool,
    pub is_array: bool,
    pub is_object: bool,
}

impl Model {
    pub fn new(name: String, schema: Schema, lang: &Lang) -> Self {
        let fields: Vec<Field> = schema
            .properties
            .unwrap_or(BTreeMap::new())
            .into_iter()
            .map(|(name, schema)| {
                lang.transform_field(Field {
                    nullable: schema.nullable.unwrap_or(false),
                    format: schema.format,
                    ref_path: schema.ref_path,
                    is_array: schema
                        .schema_type
                        .clone()
                        .into_iter()
                        .any(|t| &t == "array"),
                    r#type: schema.schema_type.expect("no field type defined"),
                    name,
                })
            })
            .collect();

        let r#type = schema.schema_type.unwrap_or("none".into());
        let is_array = &r#type == "array";
        let is_object = &r#type == "object";

        Self {
            model_name: name,
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
                    name_from_ref(&s.ref_path.clone().unwrap()).unwrap(),
                    *s,
                    lang,
                ))
            }),
            fields,
            r#type,
            is_array,
            is_object,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    pub format: Option<String>,
    pub nullable: bool,
    pub ref_path: Option<String>,
    pub is_array: bool,
}
