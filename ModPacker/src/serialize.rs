use mvutils::Savable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Savable)]
pub struct ModJson {
    pub name: String,
    pub modid: String,
    pub makers: Vec<String>,
    pub versions: ModJsonVersions,
    pub specs: ModJsonSpecs
}

#[derive(Deserialize, Savable)]
pub struct ModJsonVersions {
    pub game: String,
    pub r#mod: String,
}

#[derive(Deserialize, Savable)]
pub struct ModJsonSpecs {
    pub res: bool,
    pub targets: Vec<String>
}