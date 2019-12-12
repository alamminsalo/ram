use super::util;
use openapi::v3_0::{ObjectOrReference, Schema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ModelType {
    Primitive,
    Object,
    Array,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Model {
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
    pub extensions: HashMap<String, String>,

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
    pub fn new(name: &str, schema: &Schema) -> Self {
        let properties: Vec<Box<Model>> = schema
            .properties
            .iter()
            .flatten()
            .map(|(name, schema)| Box::new(Model::new(&name, schema)))
            .collect();

        let additional_properties: Option<Box<Model>> = schema
            .additional_properties
            .as_ref()
            .and_then(|obj_or_ref| match obj_or_ref {
                ObjectOrReference::Object(s) => Some(Box::new(Model::new("", &s))),
                _ => None,
            });

        let schema_type = schema
            .schema_type
            .as_ref()
            .unwrap_or(&String::from("object"))
            .to_owned();

        let mut model = Model {
            name: name.into(),
            ref_path: schema.ref_path.clone(),
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
            readonly: schema.read_only.unwrap_or(false),
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
            .filter(|f| f.is_primitive)
            .collect();
    }

    fn set_object_properties<'a>(&'a mut self) {
        self.object_properties = self
            .properties
            .iter()
            .cloned()
            .filter(|f| f.is_object)
            .collect()
    }

    fn set_array_properties<'a>(&'a mut self) {
        self.array_properties = self
            .properties
            .iter()
            .cloned()
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
}
