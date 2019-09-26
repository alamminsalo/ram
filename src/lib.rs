mod config;
mod lang;
mod model;

pub use config::Config;
pub use lang::Lang;
pub use model::{Field, Model};

use openapi::v3_0::{Components, ObjectOrReference::Object, Spec};
use std::path::Path;

fn generate_models(template_path: &Path, lang: Lang, components: Components) {
    let template = mustache::compile_path(template_path).unwrap();
    for (key, schema) in components.schemas.unwrap().into_iter() {
        match schema {
            Object(s) => {
                let model = Model::new(key, s, &lang);
                let render = template.render_to_string(&model).unwrap();
                // decode special characters
                let decoded = htmlescape::decode_html(&render).unwrap();
                println!("{}", &decoded);
            }
            _ => {}
        }
    }
}

pub fn generate(cfg: Config, spec: Spec) {
    let lang = cfg.get_lang().expect("failed to create lang spec!");
    generate_models(
        Path::new(&cfg.template.get("model").expect("no models template defined")),
        lang,
        spec.components.unwrap(),
    );
}
