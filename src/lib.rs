mod config;
mod model;

pub use config::{Config, Lang};
pub use model::{Model};

use openapi::v3_0::{
    ObjectOrReference::Object,
    Spec,
    Components
};
use std::path::Path;

fn gen_models_oa3(template_path: &Path, components: Components) {
    let template = mustache::compile_path(template_path).unwrap();
    for (key, schema) in components.schemas.unwrap().into_iter() {
        match schema {
            Object(s) => {
                let model = Model::new(key, s);
                println!("{}", &template.render_to_string(&model).unwrap());
            }
            _ => {}
        }
    }
}

pub fn gen_oa3(cfg: Config, spec: Spec) {
    gen_models_oa3(Path::new(&cfg.template_models), spec.components.unwrap());
}

