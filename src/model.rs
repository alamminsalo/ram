use openapi::v3_0::{Schema};
use std::collections::BTreeMap;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Model {
    pub model_name: String,
    pub fields: Vec<Field>,
    pub has_date: bool,
    pub has_datetime: bool,
}

impl Model {
    pub fn new(name: String, schema: Schema) -> Self {
        let fields: Vec<Field> = schema
            .properties
            .unwrap_or(BTreeMap::new())
            .into_iter()
            .map(|(name, schema)| Field {
                nullable: false,
                format: schema.format,
                ref_path: schema.ref_path,
                is_array: schema.schema_type.clone().into_iter().any(|t|{
                    &t == "array"
                }),
                r#type: schema.schema_type.expect("no field type defined"),
                name,
            })
            .collect();

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
            fields,
        }
    }
}

#[derive(Debug,Serialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    pub format: Option<String>,
    pub nullable: bool,
    pub ref_path: Option<String>,
    pub is_array: bool,
}
