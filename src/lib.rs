mod api;
mod config;
mod lang;
mod model;
mod state;
mod util;

pub use api::API;
pub use config::Config;
pub use lang::{ExtraFile, Lang};
pub use model::{Field, Model};
pub use state::State;

use openapi::v3_0::Spec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn generate_files(cfg: Config, spec: Spec) {
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
    util::write_files(Path::new(&models_path), render_models(&state, &lang));

    // extra files
    println!("writing extra files...");
    util::write_files_nopath(render_extra_files(&state, &lang));

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

fn render_models(state: &State, lang: &Lang) -> HashMap<String, String> {
    // compile models template
    let template_path = state.cfg.get_template("model", &lang);
    let template = mustache::compile_path(template_path).unwrap();

    // iterate components and generate models
    state
        .models
        .iter()
        .map(|model| {
            let render = template.render_to_string(&model).unwrap();
            (lang.format_filename(&model.name_lowercase), render)
        })
        .collect()
}

// Renders extra files
pub fn render_extra_files(state: &State, lang: &Lang) -> HashMap<String, String> {
    lang.files
        .iter()
        .map(|f: &ExtraFile| {
            // compile template
            let t = mustache::compile_path(&f.template).expect(&format!(
                "failed to compile extra file template: {}",
                &f.template
            ));
            // render template
            let render = t
                .render_to_string(state)
                .expect("failed to format extra file template");

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

            (path, render)
        })
        .collect()
}
