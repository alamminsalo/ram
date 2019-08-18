use ram::Config;

fn main() {
    let cfg = Config::load_file(".ram_config.yaml").unwrap();
    dbg!(&cfg);

    let spec = cfg.openapi_spec().unwrap();
    dbg!(&spec);
}
