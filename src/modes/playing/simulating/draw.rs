use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, GamemodeDrawer, RenderTargetStack},
    modes::playing::{draw_space, simulating::AdvanceMethod},
    simulator::floodfill::FloodFillError,
    utils::draw,
};

use super::ModeSimulating;

impl GamemodeDrawer for ModeSimulating {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
        use macroquad::prelude::*;

        draw_space(assets, frame_info.frames_ran as f32 / 30.0);

        self.board.draw(assets);

        // Do some janky debug textures
        for tip in self.flooder.tips.iter().flatten() {
            let (cx, cy) = self.board.coord_to_px(tip.pos);
            draw_rectangle(cx + 4.0, cy + 4.0, 8.0, 8.0, draw::hexcolor(0x00ffff_60));
        }

        for (visited, _) in self.flooder.visited.keys() {
            let (cx, cy) = self.board.coord_to_px(*visited);
            draw_rectangle(cx + 4.0, cy + 4.0, 8.0, 8.0, draw::hexcolor(0x00aa00_50));
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
        }
    }
}
