use super::helper;
use failure::Fallible;
use glob::Pattern;
use handlebars::Handlebars;
use itertools::Itertools;
use openapi::v3_0::{ObjectOrReference, Parameter, Schema, Spec};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// returns ignore patterns from '.ramignore'
fn ignore_patterns() -> Vec<Pattern> {
    fs::read_to_string(".ramignore")
        .and_then(|contents| {
            Ok(contents
                .split("\n")
                .filter_map(|line| Pattern::new(line).ok())
                .collect())
        })
        .unwrap_or(vec![])
}

// writes files in map
pub fn write_files(root: &Path, map: HashMap<PathBuf, String>) {
    let ignored = ignore_patterns();
    for (file, data) in map.iter() {
        let path = root.join(&file);
        if ignored.iter().any(|p| p.matches_path(&path)) {
            println!("ignoring file {}", path.to_str().unwrap_or(""));
            continue;
        }
        println!("writing {}", &path.to_str().unwrap());
        // create dirs if needed
        fs::create_dir_all(path.parent().expect("failed to get parent dir"))
            .expect("failed to create directory");
        fs::write(path, data).expect(&format!("failed to write file {}", &file.display()));
    }
}

// Returns model name from ref path
pub fn model_name_from_ref(ref_path: &str) -> Option<String> {
    if let Some(idx) = ref_path.rfind('/') {
        Some(ref_path[idx + 1..].to_string())
    } else {
        None
    }
}

pub fn handlebars() -> Handlebars {
    let mut hb = Handlebars::new();

    // set strict mode (fails on field not found)
    hb.set_strict_mode(true);

    // disable html escaping
    hb.register_escape_fn(handlebars::no_escape);

    // register custom helpers
    helper::register_helpers(&mut hb);

    hb
}

pub fn collect_schemas<'a>(spec: &'a Spec, root: &'a Path) -> Fallible<HashMap<String, Schema>> {
    let component_schemas = spec
        .components
        .iter()
        .flat_map(|components| {
            components
                .schemas
                .iter()
                .flatten()
                .filter_map(|(k, v)| match v {
                    ObjectOrReference::Object(t) => Some((k.clone(), t.clone())),
                    _ => None,
                })
        })
        .collect::<HashMap<String, Schema>>();

    // collect and return schemas
    Ok(iter_spec_schemas(spec)
        .flat_map(|schema| iter_ref_paths(&schema))
        .filter_map(ref_file)
        .unique()
        .flat_map(move |path| schemas_from_ref(&root, path, &HashMap::new()))
        .flatten()
        .chain(component_schemas)
        .collect())
}

pub fn collect_parameters<'a>(
    spec: &'a Spec,
    _root: &'a Path,
) -> Fallible<HashMap<String, Parameter>> {
    let component_parameters = spec
        .components
        .iter()
        .flat_map(|components| {
            components
                .parameters
                .iter()
                .flatten()
                .filter_map(|(k, v)| match v {
                    ObjectOrReference::Object(t) => Some((k.clone(), t.clone())),
                    _ => None,
                })
        })
        .collect::<HashMap<String, Parameter>>();

    Ok(component_parameters)
}

// iterates all the schemas in Spec
pub fn iter_spec_schemas<'a>(spec: &'a Spec) -> impl Iterator<Item = &'a Schema> {
    // helper function to map ObjectOrReference inner types
    fn map_obj_or_refence<'a, T: 'a>(
        iter: impl Iterator<Item = &'a ObjectOrReference<T>>,
    ) -> impl Iterator<Item = &'a T> {
        iter.filter_map(|obj_or_ref| match obj_or_ref {
            ObjectOrReference::Object(t) => Some(t),
            _ => None,
        })
    };

    let components_schemas = spec.components.iter().flat_map(|components| {
        components
            .schemas
            .iter()
            .flat_map(|hashmap| map_obj_or_refence(hashmap.values()))
    });

    let path_schemas = spec.paths.values().flat_map(move |p| {
        std::iter::empty()
            .chain(p.get.iter())
            .chain(p.put.iter())
            .chain(p.post.iter())
            .chain(p.delete.iter())
            .chain(p.options.iter())
            .chain(p.head.iter())
            .chain(p.patch.iter())
            .chain(p.trace.iter())
            .flat_map(|op| {
                // get mediatype items from both response and request
                let responses = op
                    .responses
                    .values()
                    .filter_map(|resp| resp.content.as_ref())
                    .flat_map(|r| r.values());

                let request =
                    map_obj_or_refence(op.request_body.iter()).flat_map(|b| b.content.values());

                responses.chain(request)
            })
            .flat_map(|mediatype| map_obj_or_refence(mediatype.schema.iter()))
    });

    components_schemas.chain(path_schemas)
}

/// Returns all ref_paths for schema
fn iter_ref_paths<'a>(schema: &'a Schema) -> Box<dyn Iterator<Item = &'a String> + 'a> {
    Box::new(
        schema
            .ref_path
            .iter()
            .map(|s| Some(s))
            .filter_map(|r| r)
            .chain(
                schema
                    .properties
                    .iter()
                    .flat_map(|hashmap| hashmap.values().flat_map(|s| iter_ref_paths(&s))),
            )
            .chain(schema.items.iter().flat_map(|s| iter_ref_paths(&s)))
            .chain(
                schema
                    .additional_properties
                    .iter()
                    .map(|obj_or_ref| match obj_or_ref {
                        ObjectOrReference::Object(schema) => Some(schema),
                        _ => None,
                    })
                    .filter_map(|x| x)
                    .flat_map(|s| iter_ref_paths(&s)),
            ),
    )
}

// joins a + b if b is relative path, otherwise returns b
pub fn join_relative(a: &Path, b: &Path) -> PathBuf {
    if b.is_relative() {
        a.join(b)
    } else {
        PathBuf::from(b)
    }
}

// reads schemas from file
fn read_schemas(path: &Path) -> Fallible<HashMap<String, Schema>> {
    let ext: Option<&str> = path.extension().and_then(std::ffi::OsStr::to_str);
    let data = std::fs::read_to_string(path)?;

    Ok(match ext {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&data)?,
        Some("json") => serde_json::from_str(&data)?,
        _ => failure::bail!("unsupported file type"),
    })
}

// creates file path,
// removing everything after '#'
fn ref_file<'a>(ref_path: &'a String) -> Option<&'a str> {
    ref_path
        .split("#")
        .next()
        .and_then(|p| if !p.is_empty() { Some(p) } else { None })
}

fn schemas_from_ref(
    root: &Path,
    ref_path: &str,
    a: &HashMap<String, Schema>,
) -> Fallible<HashMap<String, Schema>> {
    let mut path: PathBuf = root.join(&ref_path);

    // read schemas from file to map b,
    // filtering out schemas that are already in map a
    let b: HashMap<String, Schema> = read_schemas(&path)
        .expect("failed to read schemas!")
        .into_iter()
        .filter(|(k, _)| !a.contains_key(k))
        .collect();

    // merge together in a map
    let mut merged = a.clone();
    merged.extend(b.clone());

    // create next root path by popping filename from path
    path.pop();

    Ok(
        // fold values in map b with map c
        // (which contains now all the schemas)
        // recursively so we keep track of collected schemas so far
        b.values()
            .fold(merged, |mut acc: HashMap<String, Schema>, schema| {
                for ref_path in iter_ref_paths(&schema).filter_map(ref_file).unique() {
                    acc.extend(schemas_from_ref(&path, ref_path, &acc).unwrap());
                }

                acc
            }),
    )
}
