mod assets;
mod config;
mod helper;
mod lang;
mod model;
mod param;
mod resource;
mod state;
pub mod util;

use assets::Assets;
pub use config::Config;
pub use lang::{AddFile, Lang};
pub use model::{Model, ModelType};
pub use param::Param;
pub use resource::{GroupingStrategy, Resource, ResourceGroup};
pub use state::State;

use handlebars::Handlebars;
use openapi::v3_0::Spec;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
fn normalize_models(models: Vec<Model>) -> Vec<Model> {
    // map top-level models by name
    let models_map = models
        .iter()
        .map(|m| (m.def.clone(), m.clone()))
        .collect::<HashMap<String, Model>>();

    // normalize models
    models
        .into_iter()
        .map(|m| m.normalize(&models_map))
        .collect()
}

pub fn generate_models_v3(spec: &Spec, root: &Path) -> Vec<Model> {
    // iterate components
    // + generate models
    util::collect_schemas(spec, root)
        .expect("failed to collect schemas")
        .iter()
        .map(|(key, schema)| Model::new(key, schema, key))
        .collect()
}

pub fn generate_resources_v3(
    spec: &Spec,
    root: &Path,
    grouping_strategy: GroupingStrategy,
) -> Vec<ResourceGroup> {
    let parameters_map =
        util::collect_parameters(spec, root).expect("failed to collect parameters");
    resource::group_resources(&spec.paths, grouping_strategy, &parameters_map)
}

/// Creates ready to use state value with translated models
pub fn create_state(
    cfg: Config,
    mut models: Vec<Model>,
    mut resource_groups: Vec<ResourceGroup>,
    no_defaults: bool,
) -> State {
    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    // translate and format models and resource groups
    models = translate_models(&lang, models);
    resource_groups = translate_resource_groups(&lang, resource_groups);

    State {
        cfg,
        models,
        resource_groups,
        lang,
        no_defaults,
    }
}

pub fn generate_files(state: State) -> HashMap<PathBuf, String> {
    println!("Generating files...");
    let mut hb = Handlebars::new();
    util::init_handlebars(&mut hb);

    // add lang helpers to hb
    state.lang.add_helpers(&mut hb);

    // render files
    let files: Vec<AddFile> = state.cfg.get_files(match state.no_defaults {
        true => Some(&state.lang),
        false => None,
    });

    println!("Rendering templates...");
    render_files(&mut hb, &state, files)
}

// runs lang translations on all models
fn translate_models(lang: &Lang, models: Vec<Model>) -> Vec<Model> {
    models.into_iter().map(|m| m.translate(lang)).collect()
}

fn translate_resource_groups(
    lang: &Lang,
    resource_groups: Vec<ResourceGroup>,
) -> Vec<ResourceGroup> {
    resource_groups
        .into_iter()
        // run format on all resources
        .map(|rg| {
            let mut rg2 = rg.clone();
            rg2.resources = rg2
                .resources
                .into_iter()
                .map(|r| r.translate(lang))
                .collect();
            rg2
        })
        .collect()
}

// Renders extra files
fn render_files(
    hb: &mut Handlebars,
    state: &State,
    files: Vec<AddFile>,
) -> HashMap<PathBuf, String> {
    // state to serde json value
    let statejson = json!(&state);

    // render files
    files
        .into_iter()
        .flat_map(|f: AddFile| {
            // get data from assets and render it
            let template = Assets::read_file(&PathBuf::from(&f.template)).unwrap();
            let render = hb
                .render_template(&template, &statejson)
                .expect("failed to render additional file template");
            // make path
            let dirpath: PathBuf = if let Some(ref abspath) = f.path {
                // get from absolute path
                PathBuf::from(abspath)
            } else if let Some(ref inpath) = f.file_in {
                // get location from 'in' using config.files
                let path = state.cfg.get_path(inpath, &state.lang);
                path
            } else {
                // use rootpath
                let path = state.cfg.get_path("root", &state.lang);
                path
            };

            // If file name is defined, use it as output for file.
            // If not, then assume the filenames are found inside the templates
            match f.filename {
                Some(filename) => vec![(dirpath.join(filename), render)],
                _ => util::split_files(render, dirpath),
            }
        })
        .collect()
}
