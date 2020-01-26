use ram::{Config, GroupingStrategy};
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

    /// prints state passed to templates as json
    #[structopt(short, long)]
    debug_state: bool,
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
            let resource_groups = ram::generate_resources_v3(
                &spec,
                &specpath,
                cfg.grouping_strategy.unwrap_or(GroupingStrategy::FirstTag),
            );
            let state = ram::create_state(cfg, models, resource_groups);

            if args.debug_state {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&state).expect("failed to serialize state!")
                );
            } else {
                ram::generate_files(state, &output)
            }
        }
        _ => {
            panic!("unsupported openapi version");
        }
    };
}
