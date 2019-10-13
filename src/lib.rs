mod api;
mod assets;
mod config;
mod helper;
mod lang;
mod model;
mod state;
mod util;

pub use api::API;
use assets::Assets;
pub use config::Config;
pub use lang::{AddFile, Lang};
pub use model::{Field, Model};
pub use state::State;

use handlebars::Handlebars;
use openapi::v3_0::Spec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn generate_files(cfg: Config, spec: Spec) {
    let mut hb = util::handlebars();

    println!("generating files...");
    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    println!("generating models...");
    let models = generate_models(&cfg, &lang, &spec);

    // create state for post-processing purposes
    let state = State { cfg, models };

    // write models into specified path
    println!("writing models...");
    let models_path = state.cfg.get_path("model", &lang);
    util::write_files(
        Path::new(&models_path),
        render_models(&mut hb, &state, &lang),
    );

    // extra files
    println!("writing extra files...");
    util::write_files_nopath(render_additional_files(&mut hb, &state, &lang));

    println!("generation OK")
}

fn generate_models(cfg: &Config, lang: &Lang, spec: &Spec) -> Vec<Model> {
    // get openapi dir path
    let mut rootpath = PathBuf::from(cfg.openapi.as_ref().expect("no openapi spec defined"));
    rootpath.pop();
    // iterate components + collected schemas and generate models
    spec.collect_schemas(&rootpath)
        .expect("failed to collect schemas")
        .into_iter()
        .map(|(key, schema)| Model::new(&key, schema, &lang))
        .collect()
}

fn render_models(hb: &mut Handlebars, state: &State, lang: &Lang) -> HashMap<String, String> {
    // compile models template
    let template_path = state.cfg.get_template("model", &lang);

    // get data from assets and compile it
    let data = Assets::read_file(&template_path).unwrap();
    hb.register_template_string("model", &data)
        .expect("failed to compile models template");

    // iterate components and generate models
    state
        .models
        .iter()
        .map(|model| {
            let render = hb.render("model", &model).unwrap();
            (
                lang.format("filename", &model.filename).unwrap(),
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
