mod config;
mod lang;
mod model;

pub use config::Config;
pub use lang::Lang;
pub use model::{Field, Model};

use openapi::v3_0::{Components, ObjectOrReference::Object, Spec};
use std::path::Path;

fn gen_models_oa3(template_path: &Path, lang: Lang, components: Components) {
    let template = mustache::compile_path(template_path).unwrap();
    for (key, schema) in components.schemas.unwrap().into_iter() {
        match schema {
            Object(s) => {
                let model = Model::new(key, s, &lang);
                println!("{}", &template.render_to_string(&model).unwrap());
            }
            _ => {}
        }
    }
}

pub fn gen_oa3(cfg: Config, spec: Spec) {
    let lang = cfg.get_lang().expect("failed to create lang spec!");
    gen_models_oa3(
        Path::new(&cfg.template.model.expect("no models template defined")),
        lang,
        spec.components.unwrap(),
    );
}
