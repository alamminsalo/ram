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

pub fn generate_resources_v3(spec: &Spec, root: &Path) -> Vec<ResourceGroup> {
    let parameters_map =
        util::collect_parameters(spec, root).expect("failed to collect parameters");
    resource::group_resources(&spec.paths, GroupingStrategy::FirstTag, &parameters_map)
}

pub fn generate_files(
    cfg: Config,
    mut models: Vec<Model>,
    mut resource_groups: Vec<ResourceGroup>,
    output: &Path, // output folder
) {
    println!("generating files...");
    let mut hb = Handlebars::new();
    util::init_handlebars(&mut hb);

    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    // add lang helpers to hb
    lang.add_helpers(&mut hb);

    // translate and format models and resource groups
    models = translate_models(&lang, models);
    resource_groups = translate_resource_groups(&lang, resource_groups);

    if lang.templates.contains_key("model") {
        // write models
        println!("writing models...");
        let models_path = cfg.get_path("model", &lang);
        util::write_files(
            &output.join(&models_path),
            render_models(&mut hb, &cfg, &lang, &models),
        );
    }

    // write resources
    if cfg.templates.contains_key("resource") {
        // translate resource params
        println!("writing resources...");
        let resources_path = cfg.get_path("resource", &lang);
        util::write_files(
            &output.join(&resources_path),
            render_resources(&mut hb, &cfg, &lang, &resource_groups),
        );
    }

    // additional files
    let additional_files: Vec<AddFile> = cfg.get_additional_files(&lang);
    if !additional_files.is_empty() {
        println!("writing additional lang files...");
        let state = State {
            cfg,
            models,
            resource_groups,
        };
        util::write_files(
            &output,
            render_additional_files(&mut hb, &state, &lang, additional_files),
        );
    }

    println!("generation OK")
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

fn render_models(
    hb: &mut Handlebars,
    cfg: &Config,
    lang: &Lang,
    models: &Vec<Model>,
) -> HashMap<PathBuf, String> {
    // compile models template
    let template_path = cfg.get_template("model", &lang);

    // get data from assets and compile it
    let data = Assets::read_file(&PathBuf::from(&template_path)).unwrap();
    hb.register_template_string("model", &data)
        .expect("failed to compile models template");

    // render items
    models
        .iter()
        .map(|model| {
            let render = hb.render("model", &model).unwrap();
            (
                PathBuf::from(lang.format("filename", &model.name).unwrap()),
                htmlescape::decode_html(&render).unwrap(),
            )
        })
        .collect()
}

fn render_resources(
    hb: &mut Handlebars,
    cfg: &Config,
    lang: &Lang,
    resource_groups: &Vec<ResourceGroup>,
) -> HashMap<PathBuf, String> {
    // compile models template
    let template_path = cfg.get_template("resource", &lang);

    // get data from assets and compile it
    let data = Assets::read_file(&PathBuf::from(&template_path)).unwrap();

    hb.register_template_string("resource", &data)
        .expect("failed to compile models template");

    // render items
    resource_groups
        .iter()
        .map(|rg| {
            let render = hb.render("resource", &rg).unwrap();
            (
                PathBuf::from(lang.format("filename", &rg.name).unwrap()),
                htmlescape::decode_html(&render).unwrap(),
            )
        })
        .collect()
}

// Renders extra files
fn render_additional_files(
    hb: &mut Handlebars,
    state: &State,
    lang: &Lang,
    additional_files: Vec<AddFile>,
) -> HashMap<PathBuf, String> {
    additional_files
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
