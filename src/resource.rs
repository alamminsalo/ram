use super::param::{get_params, Param};
use super::Lang;
use itertools::Itertools;
use openapi::v3_0::{Operation, PathItem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Path params
    pub path_params: Vec<Param>,

    /// Query params
    pub query_params: Vec<Param>,
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
            path_params: get_params(op, "path"),
            query_params: get_params(op, "query"),
        }
    }

    pub fn translate(self, lang: &Lang) -> Resource {
        // translates param models
        let tr_params = |params: Vec<Param>| {
            params
                .into_iter()
                .map(|p| Param {
                    model: lang.translate(p.model),
                    ..p
                })
                .collect()
        };

        Resource {
            // also formats path
            path: lang.format_path(self.path),
            query_params: tr_params(self.query_params),
            path_params: tr_params(self.path_params),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// Group name
    /// Resources are grouped by first tag on them
    pub name: String,
    /// Resources under this group
    pub resources: Vec<Resource>,
    /// Grouping strategy used
    pub grouping_strategy: GroupingStrategy,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GroupingStrategy {
    FirstTag,
    Path,
    Separate,
}

/// Groups resources with given grouping strategy
pub fn group_resources(
    paths: &BTreeMap<String, PathItem>,
    grouping_strategy: GroupingStrategy,
) -> Vec<ResourceGroup> {
    let iter = paths.iter().flat_map(|(path, item)| {
        vec![
            (path.clone(), "GET", item.get.as_ref()),
            (path.clone(), "PUT", item.put.as_ref()),
            (path.clone(), "POST", item.post.as_ref()),
            (path.clone(), "DELETE", item.delete.as_ref()),
            (path.clone(), "OPTIONS", item.options.as_ref()),
            (path.clone(), "HEAD", item.head.as_ref()),
            (path.clone(), "PATCH", item.patch.as_ref()),
            (path.clone(), "TRACE", item.trace.as_ref()),
        ]
        .into_iter()
        .filter_map(|(path, method, op)| op.and_then(|op| Some((path, method, op))))
    });
    let strat_iter = match grouping_strategy {
        GroupingStrategy::FirstTag => iter
            .filter_map(|(path, method, op)| {
                op.tags
                    .as_ref()
                    .and_then(|tags| tags.get(0))
                    .and_then(|tag| Some((path, method, op, tag)))
            })
            .group_by(|(_, _, _, tag)| tag.clone()),
        _ => panic!("not implemented"),
    };

    // collect resourcegroups
    strat_iter
        .into_iter()
        .map(|(key, group)| ResourceGroup {
            name: key.into(),
            resources: group
                .into_iter()
                .map(|(path, method, op, _)| Resource::new(path.as_str(), method, op))
                .collect(),
            grouping_strategy,
        })
        .collect()
}
