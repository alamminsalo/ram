mod assets;
mod config;
mod helper;
mod lang;
mod model;
mod resource;
mod state;
mod util;

use assets::Assets;
pub use config::Config;
pub use lang::{AddFile, Lang};
pub use model::Model;
pub use resource::{GroupingStrategy, Resource, ResourceGroup};
pub use state::State;

use handlebars::Handlebars;
use openapi::v3_0::Spec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn generate_files(cfg: Config, spec: Spec) {
    println!("generating files...");
    let mut hb = util::handlebars();
    let mut resource_groups = vec![];

    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    // add lang helpers to hb
    lang.add_helpers(&mut hb);

    let models = generate_models(&cfg, &lang, &spec);

    if lang.templates.contains_key("model") {
        println!("generating models...");

        // write models
        println!("writing models...");
        let models_path = cfg.get_path("model", &lang);
        util::write_files(
            Path::new(&models_path),
            render_models(&mut hb, &cfg, &lang, &models),
        );
    }

    // write resources
    if cfg.templates.contains_key("resource") {
        println!("generating resource groups...");
        resource_groups = resource::group_resources(&spec.paths, GroupingStrategy::FirstTag);
        println!("writing resources...");
        let resources_path = cfg.get_path("resource", &lang);
        util::write_files(
            Path::new(&resources_path),
            render_resources(&mut hb, &cfg, &lang, &resource_groups),
        );
    }

    // additional files
    if !lang.additional_files.is_empty() {
        println!("writing additional lang files...");
        let state = State {
            cfg,
            models,
            resource_groups,
        };
        util::write_files_nopath(render_additional_files(&mut hb, &state, &lang));
    }

    println!("generation OK")
}

fn generate_models(cfg: &Config, lang: &Lang, spec: &Spec) -> Vec<Model> {
    // get openapi dir path
    let mut rootpath = PathBuf::from(cfg.openapi.as_ref().expect("no openapi spec defined"));
    rootpath.pop();
    // iterate components + collected schemas and generate models
    util::collect_schemas(spec, &rootpath)
        .expect("failed to collect schemas")
        .iter()
        .map(|(key, schema)| Model::new(key, schema))
        .map(|m| lang.translate(m))
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
    let data = Assets::read_file(&template_path).unwrap();
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
    let data = Assets::read_file(&template_path).unwrap();
    hb.register_template_string("resource", &data)
        .expect("failed to compile models template");

    // render items
    resource_groups
        .iter()
        // run format on all resources
        .map(|rg| {
            let mut rg2 = rg.clone();
            rg2.resources = rg2.resources.into_iter().map(|r| r.format(lang)).collect();
            rg2
        })
        .map(|rg| {
            let render = hb.render("resource", &rg).unwrap();
            (
                lang.format("filename", &rg.name).unwrap(),
                htmlescape::decode_html(&render).unwrap(),
            )
        })
        .collect()
}

// Renders extra files
pub fn render_additional_files(
    hb: &mut Handlebars,
    state: &State,
    lang: &Lang,
) -> HashMap<String, String> {
    lang.additional_files
        .iter()
        .chain(state.cfg.additional_files.iter())
        .map(|f: &AddFile| {
            // get data from assets and render it
            let template = Assets::read_file(&f.template).unwrap();
            let render = hb
                .render_template(&template, &state)
                .expect("failed to render additional file template");
            // make path
            let path = if let Some(ref abspath) = f.path {
                // get from absolute path
                abspath.clone()
            } else if let Some(ref inpath) = f.r#in {
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
