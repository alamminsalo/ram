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

use openapi::v3_0::{Components, ObjectOrReference::Object, Spec};
use std::collections::HashMap;
use std::path::Path;

pub fn generate_files(cfg: Config, spec: Spec) {
    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    let models = generate_models(&lang, spec.components.unwrap());

    // create state for post-processing purposes
    let state = State { cfg, models, lang };

    // write models into specified path
    let models_path = Path::new(
        state
            .cfg
            .paths
            .get("model")
            .expect("models path not defined"),
    );
    util::write_files(&models_path, render_models(&state));

    // extra files
    util::write_files_nopath(render_extra_files(&state));
}

fn generate_models(lang: &Lang, components: Components) -> Vec<Model> {
    // iterate components and generate models
    components
        .schemas
        .unwrap()
        .into_iter()
        .map(|(key, schema)| match schema {
            Object(s) => Ok(Model::new(&key, s, &lang)),
            _ => Err(()),
        })
        .filter_map(Result::ok)
        .collect()
}

fn render_models(state: &State) -> HashMap<String, String> {
    // compile models template
    let template_path = Path::new(
        state
            .cfg
            .template
            .get("model")
            .expect("no models template defined"),
    );
    let template = mustache::compile_path(template_path).unwrap();

    // iterate components and generate models
    state
        .models
        .iter()
        .map(|model| {
            let render = template.render_to_string(&model).unwrap();
            // rendering encodes special html characters, so let's decode them
            let decoded = htmlescape::decode_html(&render).unwrap();
            (state.cfg.model_path(&model, &state.lang), decoded)
        })
        .collect()
}

// Renders extra files
pub fn render_extra_files(state: &State) -> HashMap<String, String> {
    state
        .lang
        .files
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
                let dir = Path::new(state.cfg.paths.get(inpath).unwrap());
                dir.join(&f.filename).to_str().unwrap().into()
            } else {
                panic!("failed to get file render path")
            };

            (path, render)
        })
        .collect()
}
