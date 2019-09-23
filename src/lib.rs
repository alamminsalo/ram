mod config;
mod lang;
mod model;

pub use config::Config;
pub use lang::Lang;
pub use model::{Field, Model};

use openapi::v3_0::{Components, ObjectOrReference::Object, Spec};
use std::path::Path;
use handlebars::Handlebars;

fn gen_models_oa3(template_path: &Path, lang: Lang, components: Components) {
    let mut hb = Handlebars::new();
    hb.register_template_file("model", template_path).unwrap();
    let template = mustache::compile_path(template_path).unwrap();
    for (key, schema) in components.schemas.unwrap().into_iter() {
        match schema {
            Object(s) => {
                let model = Model::new(key, s, &lang);
                let render = hb.render("model", &model).unwrap();
                // let render = template.render_to_string(&model).unwrap();
                // decode special characters
                let decoded = htmlescape::decode_html(&render).unwrap();
                println!("{}", &decoded);
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
