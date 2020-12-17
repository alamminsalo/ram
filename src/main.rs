use log::{info, LevelFilter};
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
    json: bool,

    /// skips generating default asset files
    #[structopt(short, long)]
    no_defaults: bool,

    /// quiet logging level
    #[structopt(short, long)]
    quiet: bool,
}

fn init_logging(quiet: bool) {
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(true)
        .filter(
            None,
            match quiet {
                true => LevelFilter::Error,
                _ => LevelFilter::Info,
            },
        )
        .init();
}

fn main() {
    let args = Arguments::from_args();

    init_logging(args.quiet);

    let cfg = Config::load_file(&args.config).unwrap();
    let spec = openapi::from_path(&args.input).unwrap();

    let mut specpath = args.input;
    specpath.pop();

    // assemble state variable
    let state = match spec {
        openapi::OpenApi::V3_0(spec) => {
            let models = ram::generate_models_v3(&spec, &specpath);
            let resource_groups = ram::generate_resources_v3(
                &spec,
                &specpath,
                cfg.grouping_strategy.unwrap_or(GroupingStrategy::FirstTag),
            );
            ram::create_state(cfg, models, resource_groups, args.no_defaults)
        }
        _ => {
            panic!("unsupported openapi version");
        }
    };

    // output raw state as json
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&state).expect("failed to serialize state!")
        );
    }

    // if output defined, write files
    if let Some(output) = args.output {
        let files = ram::generate_files(state);
        ram::util::write_files(&output, files);
        info!("All operations finished!")
    }
}
