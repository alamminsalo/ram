use super::Field;
use failure::Fallible;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Lang {
    pub name: String,
    pub types: HashMap<String, Type>,
}

#[derive(Debug, Deserialize)]
pub struct Type {
    #[serde(default)]
    pub alias: Vec<String>,
    pub format: HashMap<String, Format>,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub r#type: String,
    pub nullable: String,
}

impl Lang {
    pub fn load_file(path: &str) -> Fallible<Self> {
        let path = Path::new(path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ext = path.extension().expect("failed to get extension");
        let ext: &str = ext.to_str().expect("failed to read extension");

        let cfg: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_reader(reader)?,
            "json" | _ => serde_json::from_reader(reader)?,
        };

        Ok(cfg)
    }

    pub fn transform_field(&self, f: Field) -> Field {
        let lang_type = self
            .types
            .iter()
            .find(|(name, t)| *name == &f.r#type || t.alias.contains(&f.r#type))
            .map(|(_, t)| t)
            .expect(&format!("couldn't find field type: {}", &f.name));

        let field_format = f.format.clone().unwrap_or("default".into());
        let lang_format = lang_type
            .format
            .get(&field_format)
            .expect("failed to find type format");

        let r#type = if f.nullable {
            lang_format.nullable.clone()
        } else {
            lang_format.r#type.clone()
        };

        Field { r#type, ..f }
    }
}
