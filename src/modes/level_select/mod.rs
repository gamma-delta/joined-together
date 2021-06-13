use cogs_gamedev::controls::InputHandler;
use macroquad::prelude::info;

use crate::{
    assets::{Assets, Level},
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, RenderTargetStack, Transition},
    controls::{Control, InputSubscriber},
    modes::ModePlaying,
    utils::{
        draw::{self, draw_space, mouse_position_pixel},
        profile::Profile,
    },
    HEIGHT, WIDTH,
};

const TEXTBOX_WIDTH: usize = 16;
const TEXTBOX_HEIGHT: usize = 9;

const CORNER_X: f32 = WIDTH / 2.0 - TEXTBOX_WIDTH as f32 * 16.0 / 2.0;
const CORNER_Y: f32 = HEIGHT / 2.0 - TEXTBOX_HEIGHT as f32 * 16.0 / 2.0;

const TEXT_OFFSET_X: f32 = 8.0;
const TEXT_OFFSET_Y: f32 = 12.0;

#[derive(Clone)]
pub struct ModeLevelSelect {
    text: String,
    advanced: bool,
}

impl ModeLevelSelect {
    pub fn new(levels: &[Level]) -> Self {
        Self {
            text: Self::get_text(levels, false),
            advanced: false,
        }
    }

    fn get_text(levels: &[Level], advanced: bool) -> String {
        let profile = Profile::get();
        let lines = (0..levels.len())
            .map(|idx| {
                let level = &levels[idx];
                let soln = profile.solutions.get(&level.filename);

                let name = if advanced {
                    &level.filename
                } else {
                    &level.name
                };
                if let Some(soln) = soln {
                    if let Some(metrics) = &soln.metrics {
                        format!(
                            "- {} ({} CYCLES, {} MIN CYCLES, {} XOVERS)",
                            name, metrics.total_cycles, metrics.min_cycles, metrics.crossovers
                        )
                    } else {
                        format!("- {}", name)
                    }
                } else {
                    format!("- {}", name)
                }
            })
            .collect::<Vec<_>>();
        format!("  LEVEL SELECT\n\n{}", lines.join("\n"))
    }

    fn get_hovered_char(&self) -> (Option<usize>, Option<usize>) {
        let (mx, my) = mouse_position_pixel();

        let offset = my - CORNER_Y - TEXT_OFFSET_Y;
        let row = offset / 6.0;
        let row = if row > 0.0 && row < 16.0 * 9.0 {
            Some(row as usize)
        } else {
            None
        };

        let offset = mx - CORNER_X - TEXT_OFFSET_X;
        let col = offset / 4.0;
        let col = if col > 0.0 && col < 16.0 * 13.0 {
            Some(col as usize)
        } else {
            None
        };

        (row, col)
    }
}

impl Gamemode for ModeLevelSelect {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        let advanced = controls.pressed(Control::Ctrl);
        if advanced != self.advanced {
            self.advanced = advanced;
            self.text = Self::get_text(&assets.levels, self.advanced);
        }

        if controls.clicked_down(Control::Select) {
            let (row, col) = self.get_hovered_char();
            info!("{:?}, {:?}", row, col);

            if let (Some(row), Some(col)) = (row, col) {
                if row >= 2 {
                    let level_idx = row - 2;
                    if let Some(new_level) = assets.levels.get(level_idx) {
                        // Maybe load a solution?
                        return Transition::Push(Box::new(ModePlaying::new(new_level, level_idx)));
                    }
                }
            }
        }

        Transition::None
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }

    fn on_resume(&mut self, assets: &Assets) {
        self.text = Self::get_text(&assets.levels, false);
    }
}

impl GamemodeDrawer for ModeLevelSelect {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
        use macroquad::prelude::*;

        draw_space(assets);

        gl_use_material(assets.shaders.hologram);
        assets
            .shaders
            .hologram
            .set_uniform("time", macroquad::time::get_time() as f32);

        draw::patch9(
            16.0,
            CORNER_X,
            CORNER_Y,
            TEXTBOX_WIDTH,
            TEXTBOX_HEIGHT,
            assets.textures.hologram_9patch,
        );

        draw::pixel_text(
            &self.text,
            CORNER_X + TEXT_OFFSET_X,
            CORNER_Y + TEXT_OFFSET_Y,
            None,
            draw::hexcolor(0xff5277_dd),
            assets,
        );

        gl_use_default_material();
    }
}
