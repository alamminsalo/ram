mod assets;
mod config;
mod helper;
mod lang;
mod model;
mod param;
mod resource;
mod state;
mod util;

use assets::Assets;
pub use config::Config;
pub use lang::{AddFile, Lang};
pub use model::{Model, ModelType};
pub use param::Param;
pub use resource::{GroupingStrategy, Resource, ResourceGroup};
pub use state::State;

use handlebars::Handlebars;
use openapi::v3_0::Spec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn generate_models_v3(spec: &Spec, root: &Path) -> Vec<Model> {
    // iterate components + collected schemas and generate models
    util::collect_schemas(spec, root)
        .expect("failed to collect schemas")
        .iter()
        .map(|(key, schema)| Model::new(key, schema))
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

pub fn generate_files(
    cfg: Config,
    mut models: Vec<Model>,
    mut resource_groups: Vec<ResourceGroup>,
    output: &Path, // output folder
) {
    println!("Generating files...");
    let mut hb = Handlebars::new();
    util::init_handlebars(&mut hb);

    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    // add lang helpers to hb
    lang.add_helpers(&mut hb);

    // translate and format models and resource groups
    models = translate_models(&lang, models);
    resource_groups = translate_resource_groups(&lang, resource_groups);

    // render files
    let files: Vec<AddFile> = cfg.get_files(&lang);
    if !files.is_empty() {
        println!("Rendering templates...");
        let state = State {
            cfg,
            models,
            resource_groups,
        };
        util::write_files(&output, render_files(&mut hb, &state, &lang, files));
    }

    println!("All operations finished!")
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
    lang: &Lang,
    files: Vec<AddFile>,
) -> HashMap<PathBuf, String> {
    files
        .into_iter()
        .flat_map(|f: AddFile| {
            // get data from assets and render it
            let template = Assets::read_file(&PathBuf::from(&f.template)).unwrap();
            let render = hb
                .render_template(&template, &state)
                .expect("failed to render additional file template");
            // make path
            let dirpath: PathBuf = if let Some(ref abspath) = f.path {
                // get from absolute path
                PathBuf::from(abspath)
            } else if let Some(ref inpath) = f.file_in {
                // get location from 'in' using config.files
                let path = state.cfg.get_path(inpath, lang);
                path
            } else {
                // use rootpath
                let path = state.cfg.get_path("root", lang);
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
