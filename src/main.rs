use ram::Config;
use std::panic;

fn main() {
    let cfg = Config::load_file(".ramconfig.yaml").unwrap();
    let spec = cfg.get_openapi().unwrap();

    match spec {
        openapi::OpenApi::V3_0(spec) => ram::generate_files(cfg, spec),
        _ => {
            panic!("unsupported openapi version");
        }
    };
}
