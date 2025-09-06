use mvengine::game::fs::smartdir::SmartDir;
use mvutils::Savable;
use crate::player::uuid::UUID;

#[derive(Clone, Debug, Savable)]
pub struct PlayerProfile {
    pub name: String,
    pub uuid: UUID
}

impl PlayerProfile {
    pub fn new() -> Self {
        Self {
            name: "Player".to_string(),
            uuid: UUID::new(),
        }
    }

    pub fn load_or_create(dir: &SmartDir) -> Self {
        if let Some(loaded) = dir.read_object::<PlayerProfile>("profile.sav") {
            loaded
        } else {
            let new = Self::new();
            dir.save_object(&new, "profile.sav");
            new
        }
    }
}