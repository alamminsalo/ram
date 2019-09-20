use ram::{Config};

fn main() {
    let cfg = Config::load_file(".ram_config.yaml").unwrap();
    let spec = cfg.openapi_spec().unwrap();


    match spec {
        openapi::OpenApi::V3_0(spec) => ram::gen_oa3(cfg,spec),
        _ =>  {}
    };
}
