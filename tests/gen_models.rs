use ram::Config;
use regex::Regex;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::panic;
use std::path::PathBuf;

#[test]
fn it_reads_models() {
    let spec = openapi::from_path("examples/openapi/petstore.yaml").unwrap();
    let specpath = PathBuf::from("examples/openapi/");
    match spec {
        openapi::OpenApi::V3_0(spec) => {
            let models = ram::generate_models_v3(&spec, &specpath);
            assert_eq!(models.len(), 4);
        }
        _ => {}
    };
}

#[test]
fn it_generates_models_rust() {
    let cfg = Config {
        lang: String::from("rust"),
        path: PathBuf::from("./tests"),
        files: vec![],
        helpers: HashMap::new(),
        paths: HashMap::new(),
    };
    let output = PathBuf::from("tests_output/models");

    let spec = openapi::from_path("examples/openapi/farm.yaml").unwrap();
    let specpath = PathBuf::from("examples/openapi/");

    // assert vars
    let models_count = 7;

    match spec {
        openapi::OpenApi::V3_0(spec) => {
            let models = ram::generate_models_v3(&spec, &specpath);
            assert_eq!(models.len(), models_count);
            ram::generate_files(cfg, models, vec![], &output)
        }
        _ => {}
    };

    // map files to name -> file
    let files: HashMap<String, DirEntry> =
        std::fs::read_dir(&PathBuf::from("tests_output/models/src/model"))
            .unwrap()
            .map(|f| {
                let f = f.unwrap();
                (f.file_name().to_str().unwrap().into(), f)
            })
            .collect();

    // assert files count: models + mod file
    assert_eq!(files.len(), models_count + 1);

    // do some regex checking
    // check that "farm.rs" contains 9 occurences of 'pub'
    let re = Regex::new(r"pub").unwrap();
    let contents: String = std::fs::read_to_string(files.get("farm.rs").unwrap().path()).unwrap();

    assert_eq!(re.find_iter(&contents).count(), 9);
}
