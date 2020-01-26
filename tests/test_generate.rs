use inflector::Inflector;
use ram::{Config, GroupingStrategy};
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
        grouping_strategy: None,
    };
    let output = PathBuf::from("tests_output/models");

    let spec = openapi::from_path("examples/openapi/farm.yaml").unwrap();
    let specpath = PathBuf::from("examples/openapi/");

    // assert vars
    let models_count = 7;
    let mut models = vec![];

    match spec {
        openapi::OpenApi::V3_0(spec) => {
            models = ram::generate_models_v3(&spec, &specpath);
            assert_eq!(models.len(), models_count);
            ram::generate_files(cfg, models.clone(), vec![], &output)
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

    // gather some variables from models
    for model in models {
        let props_iter = model.properties.iter().chain(
            model
                .additional_properties
                .iter()
                .flat_map(|p| &p.properties),
        );

        // pub should occur in field names and struct def
        let count_pub = props_iter.clone().count() + 1;
        let count_i32 = props_iter
            .clone()
            .filter(|p| {
                p.schema_type == "integer"
                    || (p.schema_type == "array"
                        && match p.items.as_ref() {
                            Some(item) => item.schema_type == "integer",
                            _ => false,
                        })
            })
            .count();
        let count_option = props_iter
            .clone()
            .filter(|p| {
                p.nullable
                    || (p.schema_type == "array"
                        && match p.items.as_ref() {
                            Some(item) => item.nullable,
                            _ => false,
                        })
            })
            .count();
        let count_box = props_iter
            .clone()
            .filter(|p| {
                p.schema_type == "object"
                    || (p.schema_type == "array"
                        && match p.items.as_ref() {
                            Some(item) => item.schema_type == "object",
                            _ => false,
                        })
            })
            .count();
        let count_vec = props_iter
            .clone()
            .filter(|p| p.schema_type == "array")
            .count();

        println!("counted test variables!");

        // do some regex checking
        // check that "model.rs" contains 9 occurences of 'pub'
        let contents: String = std::fs::read_to_string(
            files
                .get(&format!("{}.rs", &model.name.to_snake_case()))
                .unwrap()
                .path(),
        )
        .unwrap();

        println!("read file contents");

        assert_eq!(
            Regex::new(r"pub").unwrap().find_iter(&contents).count(),
            count_pub
        );
        assert_eq!(
            Regex::new(r"i32").unwrap().find_iter(&contents).count(),
            count_i32
        );
        assert_eq!(
            Regex::new(r"Box<.+>").unwrap().find_iter(&contents).count(),
            count_box
        );
        assert_eq!(
            Regex::new(r"Vec<.+>").unwrap().find_iter(&contents).count(),
            count_vec
        );
        assert_eq!(
            Regex::new(r"Option<.+>")
                .unwrap()
                .find_iter(&contents)
                .count(),
            count_option
        );
    }
}

#[test]
fn it_generates_resources_rust() {
    let cfg = Config::load_file(&PathBuf::from("examples/rust/rocket/rocket.yaml")).unwrap();
    let output = PathBuf::from("tests_output/res");

    let spec = openapi::from_path("examples/openapi/farm.yaml").unwrap();
    let specpath = PathBuf::from("examples/openapi/");

    // assert vars
    let res_count = 1;
    let mut resource_groups = vec![];

    match spec {
        openapi::OpenApi::V3_0(spec) => {
            resource_groups = ram::generate_resources_v3(
                &spec,
                &specpath,
                cfg.grouping_strategy.unwrap_or(GroupingStrategy::FirstTag),
            );
            assert_eq!(resource_groups.len(), res_count);
            ram::generate_files(cfg, vec![], resource_groups.clone(), &output)
        }
        _ => {}
    };

    // map files to name -> file
    let files: HashMap<String, DirEntry> =
        std::fs::read_dir(&PathBuf::from("tests_output/res/src/api"))
            .unwrap()
            .map(|f| {
                let f = f.unwrap();
                (f.file_name().to_str().unwrap().into(), f)
            })
            .collect();

    // assert files count: res_count + mod file
    assert_eq!(files.len(), res_count + 1);

    // gather some variables from models
    for group in resource_groups {
        // do some regex checking
        let contents: String = std::fs::read_to_string(
            files
                .get(&format!("{}.rs", &group.name.to_snake_case()))
                .unwrap()
                .path(),
        )
        .unwrap();

        assert_eq!(
            Regex::new(r"#\[get(.+)\]")
                .unwrap()
                .find_iter(&contents)
                .count(),
            group.resources.iter().filter(|r| r.method == "GET").count()
        );

        assert_eq!(
            Regex::new(r"Box<.+>").unwrap().find_iter(&contents).count(),
            group
                .resources
                .iter()
                .filter(|r| r.responses.get("200").is_some())
                .count()
        );
    }
}
