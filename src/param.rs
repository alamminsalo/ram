use super::Model;
use openapi::v3_0::{ObjectOrReference, Operation, Parameter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub model: Model,
    pub required: bool,
}

fn from_param(p: &Parameter) -> Option<Param> {
    p.schema.as_ref().and_then(|schema| {
        Some(Param {
            name: p.name.clone(),
            model: Model::new(&p.name, &schema),
            required: p.required.unwrap_or(false),
        })
    })
}

/// Returns (path, query) parameter lists
pub fn get_params(operation: &Operation, location: &str) -> Vec<Param> {
    operation
        .parameters
        .iter()
        .flat_map(|params| {
            params.iter().filter_map(|p| match p {
                ObjectOrReference::Object(t) => Some(t),
                _ => None,
            })
        })
        .filter(|p| p.location == location)
        .filter_map(from_param)
        .collect()
}
