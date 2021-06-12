#![allow(clippy::eval_order_dependence)]

use macroquad::{
    audio::{load_sound, Sound},
    miniquad::*,
    prelude::*,
};
use once_cell::sync::Lazy;

use std::path::PathBuf;

pub struct Assets {
    pub textures: Textures,
    pub sounds: Sounds,
    pub shaders: Shaders,
}

impl Assets {
    pub async fn init() -> Self {
        Self {
            textures: Textures::init().await,
            sounds: Sounds::init().await,
            shaders: Shaders::init().await,
        }
    }
}

pub struct Textures {
    pub title_banner: Texture2D,
    pub font: Texture2D,

    pub cable_atlas: Texture2D,
    pub port_atlas: Texture2D,
    pub error_atlas: Texture2D,

    pub hologram_9patch: Texture2D,
    pub you_win: Texture2D,
}

impl Textures {
    async fn init() -> Self {
        Self {
            title_banner: texture("title/banner").await,
            font: texture("ui/font").await,

            cable_atlas: texture("cable_atlas").await,
            port_atlas: texture("port_atlas").await,
            error_atlas: texture("error_atlas").await,

            hologram_9patch: texture("ui/hologram_9patch").await,
            you_win: texture("ui/you_win").await,
        }
    }
}

pub struct Sounds {
    pub title_jingle: Sound,
}

impl Sounds {
    async fn init() -> Self {
        Self {
            title_jingle: sound("title/jingle").await,
        }
    }
}

pub struct Shaders {
    pub space: Material,
    pub hologram: Material,
}

impl Shaders {
    async fn init() -> Self {
        Self {
            space: material_vert_frag(
                "standard",
                "space",
                MaterialParams {
                    textures: Vec::new(),
                    uniforms: vec![(String::from("time"), UniformType::Float1)],
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        ..Default::default()
                    },
                },
            )
            .await,
            hologram: material_vert_frag(
                "standard",
                "hologram",
                MaterialParams {
                    textures: Vec::new(),
                    uniforms: vec![(String::from("time"), UniformType::Float1)],
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        ..Default::default()
                    },
                },
            )
            .await,
        }
    }
}

/// Path to the assets root
static ASSETS_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_arch = "wasm32") {
        PathBuf::from("./assets")
    } else if cfg!(debug_assertions) {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
    } else {
        todo!("assets path for release hasn't been finalized yet ;-;")
    }
});

async fn texture(path: &str) -> Texture2D {
    let with_extension = path.to_owned() + ".png";
    let tex = load_texture(
        ASSETS_ROOT
            .join("textures")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    tex.set_filter(FilterMode::Nearest);
    tex
}

async fn sound(path: &str) -> Sound {
    let with_extension = path.to_owned() + ".ogg";
    load_sound(
        ASSETS_ROOT
            .join("sounds")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap()
}

async fn material_vert_frag(vert_stub: &str, frag_stub: &str, params: MaterialParams) -> Material {
    let full_stub = ASSETS_ROOT.join("shaders");
    let vert = load_string(
        full_stub
            .join(vert_stub)
            .with_extension("vert")
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    let frag = load_string(
        full_stub
            .join(frag_stub)
            .with_extension("frag")
            .to_string_lossy()
            .as_ref(),
    )
    .await
    .unwrap();
    load_material(&vert, &frag, params).unwrap()
}

async fn material(path_stub: &str, params: MaterialParams) -> Material {
    material_vert_frag(path_stub, path_stub, params).await
}
