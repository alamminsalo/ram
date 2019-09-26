mod config;
mod lang;
mod model;
mod util;

pub use config::Config;
pub use lang::Lang;
pub use model::{Field, Model};

use openapi::v3_0::{Components, ObjectOrReference::Object, Spec};
use std::collections::HashMap;
use std::path::Path;

fn generate_models(cfg: &Config, components: Components) -> HashMap<String, String> {
    // compile models template
    let template_path = Path::new(
        cfg.template
            .get("model")
            .expect("no models template defined"),
    );
    let template = mustache::compile_path(template_path).unwrap();

    // get lang config
    let lang = cfg.get_lang().expect("failed to create lang spec!");

    // iterate components and generate models
    components
        .schemas
        .unwrap()
        .into_iter()
        .map(|(key, schema)| {
            match schema {
                Object(s) => {
                    let model = Model::new(&key, s, &lang);
                    let render = template.render_to_string(&model).unwrap();
                    // rendering encodes special html characters, so let's decode them
                    let decoded = htmlescape::decode_html(&render).unwrap();
                    Ok((cfg.model_path(&model, &lang), decoded))
                }
                _ => Err(()),
            }
        })
        .filter_map(Result::ok)
        .collect()
}

pub fn generate(cfg: Config, spec: Spec) {
    let models = generate_models(&cfg, spec.components.unwrap());
    util::write_files(models);
}
