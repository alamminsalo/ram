use super::param::{get_params_operation, get_params_path, Param};
use super::Lang;
use super::Model;
use indexmap::IndexMap;
use itertools::Itertools;
use openapi::v3_0::ObjectOrReference;
use openapi::v3_0::{Operation, Parameter, PathItem};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

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

    /// Result
    pub responses: HashMap<String, Model>,
}

impl Resource {
    pub fn new(
        path: &str,
        method: &str,
        op: &Operation,
        parameters: &HashMap<String, Parameter>,
        mut path_params: Vec<Param>,
        mut query_params: Vec<Param>,
    ) -> Resource {
        // extend route params with local method params
        path_params.extend(get_params_operation(op, "path", parameters));
        query_params.extend(get_params_operation(op, "query", parameters));

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
            path_params,
            query_params,
            responses: op
                // Take 200 application/json from response content and apply that as type
                .responses
                .iter()
                .filter_map(|(code, resp)| {
                    resp.content.as_ref().and_then(|contentmap| {
                        contentmap.get("application/json").and_then(|mediatype| {
                            match &mediatype.schema {
                                Some(ObjectOrReference::Object(schema)) => {
                                    Some((code.clone(), Model::new("", &schema)))
                                }
                                _ => None,
                            }
                        })
                    })
                })
                .collect(),
        }
    }

    pub fn translate(self, lang: &Lang) -> Resource {
        let tr_params = |params: Vec<Param>| {
            params
                .into_iter()
                .map(|p| Param {
                    model: p.model.translate(lang),
                    ..p
                })
                .collect()
        };

        Resource {
            // also formats path
            path: lang.format_path(self.path),
            query_params: tr_params(self.query_params),
            path_params: tr_params(self.path_params),
            responses: self
                .responses
                .into_iter()
                .map(|(key, model)| (key, model.translate(lang)))
                .collect(),
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
    Nothing,
    Path,
    FirstTag,
    Operation,
}

/// Groups resources with given grouping strategy
pub fn group_resources(
    paths: &IndexMap<String, PathItem>,
    grouping_strategy: GroupingStrategy,
    parameters: &HashMap<String, Parameter>,
) -> Vec<ResourceGroup> {
    let iter = paths.iter().flat_map(|(path, item)| {
        let path_params = get_params_path(item, "path", parameters);
        let query_params = get_params_path(item, "query", parameters);
        vec![
            (
                path.clone(),
                "GET",
                item.get.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "PUT",
                item.put.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "POST",
                item.post.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "DELETE",
                item.delete.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "OPTIONS",
                item.options.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "HEAD",
                item.head.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "PATCH",
                item.patch.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
            (
                path.clone(),
                "TRACE",
                item.trace.as_ref(),
                path_params.clone(),
                query_params.clone(),
            ),
        ]
        .into_iter()
        .filter_map(|(path, method, op, path_params, query_params)| {
            op.and_then(|op| Some((path, method, op, path_params, query_params)))
        })
    });
    let strat_iter = iter.filter_map(|(path, method, op, path_params, query_params)| {
        match grouping_strategy {
            // everything is in same group
            GroupingStrategy::Nothing => {
                Some(("".into(), path, method, op, path_params, query_params))
            }

            // groups by path
            GroupingStrategy::Path => {
                Some((path.clone(), path, method, op, path_params, query_params))
            }

            // groups by first tag
            GroupingStrategy::FirstTag => op
                .tags
                .as_ref()
                .and_then(|tags| tags.get(0))
                .and_then(|tag| Some((tag.clone(), path, method, op, path_params, query_params))),

            // groups by operation id
            GroupingStrategy::Operation => op.operation_id.as_ref().and_then(|operationid| {
                Some((
                    operationid.clone(),
                    path,
                    method,
                    op,
                    path_params,
                    query_params,
                ))
            }),
        }
    });

    // collect resourcegroups
    strat_iter
        .group_by(|(key, _, _, _, _, _)| key.clone())
        .into_iter()
        .map(|(key, group)| ResourceGroup {
            name: key.into(),
            resources: group
                .into_iter()
                .map(|(_, path, method, op, path_params, query_params)| {
                    Resource::new(
                        path.as_str(),
                        method,
                        op,
                        parameters,
                        path_params,
                        query_params,
                    )
                })
                .collect(),
            grouping_strategy,
        })
        .collect()
}
