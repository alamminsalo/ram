use super::{Config, Lang, Model, ResourceGroup};
use serde::{Deserialize, Serialize};

// full model generation state, to contain processed models and apis
#[derive(Debug, Deserialize, Serialize)]
pub struct State {
    pub models: Vec<Model>,
    pub cfg: Config,
    pub lang: Lang,
    pub resource_groups: Vec<ResourceGroup>,
    pub no_defaults: bool,
}
