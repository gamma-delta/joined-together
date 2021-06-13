use cogs_gamedev::ease::Interpolator;

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, GamemodeDrawer, RenderTargetStack},
    modes::playing::{draw_space, simulating::AdvanceMethod},
    simulator::floodfill::FloodFillError,
    utils::draw,
    HEIGHT, WIDTH,
};

use super::ModeSimulating;

impl GamemodeDrawer for ModeSimulating {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
        use macroquad::prelude::*;

        draw_space(assets);

        self.board.draw(assets);

        // Do some janky debug textures
        for tip in self.flooder.tips.iter().flatten() {
            let (cx, cy) = self.board.coord_to_px(tip.pos);
            draw_rectangle(cx + 4.0, cy + 4.0, 8.0, 8.0, draw::hexcolor(0x00ffff_cc));
        }

        for (visited, _) in self.flooder.visited.keys() {
            let (cx, cy) = self.board.coord_to_px(*visited);
            draw_rectangle(cx + 4.0, cy + 4.0, 8.0, 8.0, draw::hexcolor(0x990000_99));
        }

        if let AdvanceMethod::Errors(errs) = &self.advance_method {
            for error in errs {
                let pos = match error {
                    FloodFillError::BadCableKind(pos)
                    | FloodFillError::NoEntrance(pos)
                    | FloodFillError::SpilledIntoSpace(pos)
                    | FloodFillError::Backtrack(pos)
                    | FloodFillError::BadOutput(pos, _) => *pos,
                };
                let (cx, cy) = self.board.coord_to_px(pos);
                let cx = cx + 8.0;
                let cy = cy + 8.0;

                // draw arrow
                draw_texture_ex(
                    assets.textures.error_atlas,
                    cx,
                    cy,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(80.0, 0.0, 16.0, 16.0)),
                        ..Default::default()
                    },
                );

                let sx = match error {
                    FloodFillError::BadCableKind(_) => 0.0,
                    FloodFillError::NoEntrance(_) => 16.0,
                    FloodFillError::SpilledIntoSpace(_) => 32.0,
                    FloodFillError::Backtrack(_) => 48.0,
                    FloodFillError::BadOutput(_, _) => 64.0,
                };
                draw_texture_ex(
                    assets.textures.error_atlas,
                    cx + 4.0,
                    cy + 4.0,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, 0.0, 16.0, 16.0)),
                        ..Default::default()
                    },
                );
            }
        } else if let AdvanceMethod::WinScreen {
            appear_progress,
            text,
        } = &self.advance_method
        {
            let patch_width = 7;
            let patch_height = 4;

            // Get the origin X/Y
            let ox = WIDTH / 2.0 - (16.0 * patch_width as f32) / 2.0;
            let oy = appear_progress.quad_out(
                patch_height as f32 * -6.0,
                HEIGHT / 2.0 - (16.0 * patch_height as f32) / 2.0 - 48.0,
            );

            gl_use_material(assets.shaders.hologram);
            assets
                .shaders
                .hologram
                .set_uniform("time", frame_info.frames_ran as f32 / 30.0);

            draw::patch9(
                16.0,
                ox,
                oy,
                patch_width,
                patch_height,
                assets.textures.hologram_9patch,
            );

            draw_texture(
                assets.textures.you_win,
                WIDTH / 2.0 - assets.textures.you_win.width() / 2.0,
                oy + 9.0,
                WHITE,
            );

            draw::pixel_text(
                text,
                ox + 5.0,
                oy + 18.0,
                None,
                draw::hexcolor(0xff5277_dd),
                assets,
            );

            gl_use_default_material();
        }
    }
}
