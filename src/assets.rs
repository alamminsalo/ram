use failure::{format_err, Error};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

impl Assets {
    /// Tries to read file first from fs
    /// then from bundled assets
    pub fn read_file(path: &str) -> Result<String, Error> {
        std::fs::read_to_string(path).or_else(|_| {
            Self::get(path)
                .and_then(|cow| Some(cow.into_owned()))
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .ok_or(format_err!("failed to read asset: {}", path))
        })
    }
}
