use super::util;
use super::Model;
use openapi::v3_0::{ObjectOrReference, Operation, Parameter, PathItem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Returns parameter lists by location
pub fn get_params_operation(
    operation: &Operation,
    location: &str,
    parameters: &HashMap<String, Parameter>,
) -> Vec<Param> {
    operation
        .parameters
        .iter()
        .flat_map(|params| {
            params.iter().filter_map(|p| match p {
                ObjectOrReference::Object(t) => Some(t),
                ObjectOrReference::Ref { ref_path } => {
                    util::model_name_from_ref(&ref_path).and_then(|name| parameters.get(&name))
                }
            })
        })
        .filter(|p| p.location == location)
        .filter_map(from_param)
        .collect()
}

/// Returns parameter lists by location
pub fn get_params_path(
    path: &PathItem,
    location: &str,
    parameters: &HashMap<String, Parameter>,
) -> Vec<Param> {
    path.parameters
        .iter()
        .flat_map(|params| {
            params.iter().filter_map(|p| match p {
                ObjectOrReference::Object(t) => Some(t),
                ObjectOrReference::Ref { ref_path } => {
                    dbg!(&ref_path);
                    util::model_name_from_ref(&ref_path).and_then(|name| parameters.get(&name))
                }
            })
        })
        .filter(|p| p.location == location)
        .filter_map(from_param)
        .collect()
}
