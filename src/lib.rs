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
pub use model::Model;
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
    let mut hb = util::handlebars();

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

fn translate_models(lang: &Lang, models: Vec<Model>) -> Vec<Model> {
    // runs lang translations on all models
    models.into_iter().map(|m| lang.translate(m)).collect()
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
) -> HashMap<String, String> {
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
                lang.format("filename", &model.name).unwrap(),
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
) -> HashMap<String, String> {
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
            // // translates param models
            // let tr_params = |params: Vec<Param>| {
            //     params
            //         .into_iter()
            //         .map(|p| Param {
            //             model: lang.translate(p.model),
            //             ..p
            //         })
            //         .collect()
            // };
            // let resources: Vec<Resource> = rg
            //     .resources
            //     .iter()
            //     .cloned()
            //     .map(|resource| Resource {
            //         query_params: tr_params(resource.query_params),
            //         path_params: tr_params(resource.path_params),
            //         ..resource
            //     })
            //     .collect();

            // let resourcegroup = ResourceGroup {
            //     name: rg.name.clone(),
            //     grouping_strategy: rg.grouping_strategy,
            //     resources,
            // };

            let render = hb.render("resource", &rg).unwrap();
            (
                lang.format("filename", &rg.name).unwrap(),
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
) -> HashMap<String, String> {
    additional_files
        .into_iter()
        .map(|f: AddFile| {
            // get data from assets and render it
            let template = Assets::read_file(&PathBuf::from(&f.template)).unwrap();
            let render = hb
                .render_template(&template, &state)
                .expect("failed to render additional file template");
            // make path
            let path = if let Some(ref abspath) = f.path {
                // get from absolute path
                abspath.clone()
            } else if let Some(ref inpath) = f.file_in {
                // get location from 'in' using config.files
                let path = state.cfg.get_path(inpath, lang);
                let dir = Path::new(&path);
                dir.join(&f.filename).to_str().unwrap().into()
            } else {
                panic!("failed to get file render path")
            };

            (path, htmlescape::decode_html(&render).unwrap())
        })
        .collect()
}
