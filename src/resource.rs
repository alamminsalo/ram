use openapi::v3_0::{Operation, PathItem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI
    pub path: String,

    /// HTTP method
    pub method: String,

    /// Resource name usable for function names
    pub name: String,

    /// Short summary
    pub summary: Option<String>,

    /// Resource description
    pub description: Option<String>,
}

impl Resource {
    pub fn new(path: &str, method: &str, op: &Operation) -> Resource {
        Resource {
            path: path.into(),
            method: method.into(),
            name: op
                .operation_id
                .as_ref()
                .expect("missing operation_id on resource")
                .clone(),
            summary: op.summary.clone(),
            description: op.description.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// Group name
    /// Resources are grouped by first tag on them
    pub name: String,
    /// Resources under this group
    pub resources: Vec<Resource>,
}

impl ResourceGroup {
    pub fn new(name: &str, paths: BTreeMap<&str, &PathItem>) -> ResourceGroup {
        ResourceGroup {
            name: name.into(),
            resources: paths
                .iter()
                .flat_map(|(path, item)| {
                    [
                        ("GET", item.get.as_ref()),
                        ("PUT", item.put.as_ref()),
                        ("POST", item.post.as_ref()),
                        ("DELETE", item.delete.as_ref()),
                        ("OPTIONS", item.options.as_ref()),
                        ("HEAD", item.head.as_ref()),
                        ("PATCH", item.patch.as_ref()),
                        ("TRACE", item.trace.as_ref()),
                    ]
                    .into_iter()
                    .filter(|(_, op)| op.is_some())
                    .map(move |(method, op)| Resource::new(&path, method, &op.unwrap()))
                    .collect::<Vec<Resource>>()
                })
                .collect(),
        }
    }
}
