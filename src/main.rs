use ram::{Config,Model};
use openapi::OpenApi;
use openapi::v3_0::ObjectOrReference::{Object};

fn main() {
    let cfg = Config::load_file(".ram_config.yaml").unwrap();

    let spec = cfg.openapi_spec().unwrap();
    match spec {
        OpenApi::V3_0(spec) => {
            let components  = spec.components.unwrap();
            for (key, schema) in components.schemas.unwrap().into_iter() {
                match schema {
                    Object(s) => {
                        let model = Model::new(key, s);
                        dbg!(&model);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };

}
