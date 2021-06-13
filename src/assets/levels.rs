use macroquad::prelude::load_string;
use serde::Deserialize;

use crate::simulator::board::Board;

use super::ASSETS_ROOT;

pub(super) async fn get_levels() -> Vec<Level> {
    let manifest = load_string(
        ASSETS_ROOT
            .join("levels/manifest.txt")
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();

    let mut levels = Vec::new();
    for line in manifest.lines() {
        let file = load_string(
            ASSETS_ROOT
                .join("levels")
                .join(line)
                .with_extension("json")
                .to_string_lossy()
                .as_ref(),
        )
        .await
        .unwrap();
        let mut level: Level = serde_json::from_str(&file).unwrap();
        level.filename = line.to_owned();
        levels.push(level);
    }

    levels
}

#[derive(Deserialize)]
pub struct Level {
    #[serde(default)]
    pub filename: String,
    pub name: String,

    #[serde(flatten)]
    pub starting_board: Board,
}
