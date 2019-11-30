use ram::Config;
use std::panic;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ram", about = "openapi generator")]
struct Arguments {
    /// ram configuration file path
    #[structopt(short, long)]
    config: PathBuf,

    /// input openapi spec file
    #[structopt(short, long)]
    input: PathBuf,

    /// output path
    #[structopt(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Arguments::from_args();
    let cfg = Config::load_file(&args.config).unwrap();
    let spec = openapi::from_path(&args.input).unwrap();
    let output = args.output.unwrap_or(PathBuf::from("./"));

    let mut specpath = args.input;
    specpath.pop();

    match spec {
        openapi::OpenApi::V3_0(spec) => {
            let models = ram::generate_models_v3(&spec, &specpath);
            let resource_groups = ram::generate_resources_v3(&spec, &specpath);
            ram::generate_files(cfg, models, resource_groups, &output)
        }
        _ => {
            panic!("unsupported openapi version");
        }
    };
}
