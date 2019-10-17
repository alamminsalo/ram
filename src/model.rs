use super::lang::Lang;
use super::util;
use inflector::Inflector;
use openapi::v3_0::{ObjectOrReference, Schema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Model {
    pub name: String,
    pub name_lowercase: String,
    pub name_pascalcase: String,
    pub name_snakecase: String,
    pub filename: String,
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
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Field {
//     pub name: String,
//     pub r#type: String,
//     pub format: Option<String>,
//     pub nullable: bool,
//     pub ref_path: Option<String>,
//     pub is_array: bool,
// }

impl Model {
    pub fn new(name: &str, schema: &Schema, lang: &Lang) -> Self {
        let fields: Vec<Box<Model>> = schema
            .properties
            .iter()
            .flatten()
            .map(|(name, schema)| {
                // translate using language spec
                // {
                //     nullable: schema.nullable.unwrap_or(false),
                //     format: schema.format,
                //     ref_path: schema.ref_path,
                //     is_array: schema
                //         .schema_type
                //         .clone()
                //         .into_iter()
                //         .any(|t| &t == "array"),
                //     r#type: schema.schema_type.unwrap_or("object".into()),
                //     name,
                // })
                Box::new(lang.translate(Model::new(&name, schema, lang)))
            })
            .collect();

        let additional_fields: Option<Box<Model>> =
            schema
                .additional_properties
                .as_ref()
                .and_then(|obj_or_ref| match obj_or_ref {
                    ObjectOrReference::Object(s) => Some(Box::new(Model::new("", &s, &lang))),
                    _ => None,
                });

        let r#type = schema
            .schema_type
            .as_ref()
            .unwrap_or(&String::from("object"))
            .to_owned();
        let is_array = &r#type == "array";
        let is_object = &r#type == "object";

        Self {
            // TODO: sense
            name: lang
                .format("reserved", &name.to_string())
                .unwrap_or(name.into()),
            name_lowercase: lang
                .format("reserved", &name.to_lowercase())
                .unwrap_or(name.to_lowercase()),
            name_pascalcase: lang
                .format("reserved", &name.to_pascal_case())
                .unwrap_or(name.to_pascal_case()),
            name_snakecase: lang
                .format("reserved", &name.to_snake_case())
                .unwrap_or(name.to_snake_case()),
            filename: name.to_snake_case(),
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
                        lang.format(
                            "classname",
                            &util::model_name_from_ref(&ref_path)
                                .expect("failed to get model name from ref_path"),
                        )
                        .expect("failed to format classname")
                    })
                    .unwrap_or(String::new());
                Box::new(lang.translate(Model::new(&name, &s, lang)))
            }),
            nullable: schema.nullable.unwrap_or(false),
            description: schema.description.clone(),
            format: schema.format.clone(),
            fields,
            additional_fields,
            r#type,
            is_array,
            is_object,
        }
    }
}
