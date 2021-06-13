use cogs_gamedev::{
    ease::Interpolator,
    grids::{Direction4, Rotation},
};

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, GamemodeDrawer, RenderTargetStack},
    modes::playing::{
        draw_space,
        simulating::{AdvanceMethod, STEP_TIME_ON_DEMAND},
    },
    simulator::{
        floodfill::FloodFillError,
        transport::{Cable, CableKind},
    },
    utils::draw::{self, hexcolor},
    HEIGHT, WIDTH,
};

use super::ModeSimulating;

impl GamemodeDrawer for ModeSimulating {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
        use macroquad::prelude::*;

        draw_space(assets);

        self.board.draw(assets);

        gl_use_material(assets.shaders.cables);

        // Deal with crossovers later by putting all horizontals before verticals
        let mut crossovers = Vec::new();

        let dt = frame_info.frames_ran - self.step_start;
        let step_time = match self.advance_method {
            AdvanceMethod::ByFrames { interval, .. } => interval,
            AdvanceMethod::OnDemand => STEP_TIME_ON_DEMAND,
            // whatever
            _ => 1,
        };
        let tip_progress = (dt as f32 / step_time as f32).clamp(0.0, 1.0);

        for tip in self.flooder.tips.iter().flatten() {
            if let Some(cable) = self.board.cables.get(&tip.pos) {
                let col = tip.resource.color();
                // On straight: progress left->right top->bottom
                // On bent: ccw->cw
                let (progress, kind) = match cable {
                    Cable::Straight { kind, horizontal } => {
                        if *horizontal == tip.facing.is_horizontal() {
                            if matches!(tip.facing, Direction4::East | Direction4::South) {
                                (tip_progress, *kind)
                            } else {
                                (-tip_progress, *kind)
                            }
                        } else {
                            // make up a kind
                            (0.0, CableKind::Wire)
                        }
                    }
                    Cable::Bent { kind, ccw_dir } => {
                        if tip.facing.flip() == *ccw_dir {
                            (tip_progress, *kind)
                        } else if tip.facing.flip() == ccw_dir.rotate(Rotation::Clockwise) {
                            (-tip_progress, *kind)
                        } else {
                            (0.0, CableKind::Wire)
                        }
                    }
                    Cable::Crossover {
                        horiz_kind,
                        vert_kind,
                    } => {
                        let kind = if tip.facing.is_horizontal() {
                            *horiz_kind
                        } else {
                            *vert_kind
                        };
                        let progress = if matches!(tip.facing, Direction4::East | Direction4::South)
                        {
                            tip_progress
                        } else {
                            -tip_progress
                        };

                        crossovers.push((kind, tip.pos, tip.facing, &tip.resource, progress));
                        continue;
                    }
                };

                assets
                    .shaders
                    .cables
                    .set_uniform("progress", [col.r, col.g, col.b, progress]);
                assets
                    .shaders
                    .cables
                    .set_uniform("isPipe", if kind == CableKind::Pipe { 1i32 } else { 0 });

                let (cx, cy) = self.board.coord_to_px(tip.pos);
                let ((sx, sy), _) = cable.get_slices();
                draw_texture_ex(
                    assets.textures.cable_atlas,
                    cx,
                    cy,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, sy + 32.0, 16.0, 16.0)),
                        ..Default::default()
                    },
                );
            }
        }

        for ((visited, horiz), resource) in self.flooder.visited.iter() {
            if let Some(cable) = self.board.cables.get(visited) {
                let (progress, kind) = match cable {
                    Cable::Straight { kind, horizontal } => {
                        if *horizontal == *horiz {
                            (1.0, *kind)
                        } else {
                            // make up a kind
                            (0.0, CableKind::Wire)
                        }
                    }
                    Cable::Bent { kind, ccw_dir } => (1.0, *kind),
                    Cable::Crossover {
                        horiz_kind,
                        vert_kind,
                    } => {
                        let kind = if *horiz { *horiz_kind } else { *vert_kind };
                        crossovers.push((
                            kind,
                            *visited,
                            if *horiz {
                                Direction4::East
                            } else {
                                Direction4::North
                            },
                            resource,
                            1.0,
                        ));
                        continue;
                    }
                };

                let col = resource.color();

                assets
                    .shaders
                    .cables
                    .set_uniform("progress", [col.r, col.g, col.b, progress]);
                assets
                    .shaders
                    .cables
                    .set_uniform("isPipe", if kind == CableKind::Pipe { 1i32 } else { 0 });

                let (cx, cy) = self.board.coord_to_px(*visited);
                let ((sx, sy), _) = cable.get_slices();
                draw_texture_ex(
                    assets.textures.cable_atlas,
                    cx,
                    cy,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, sy + 32.0, 16.0, 16.0)),
                        ..Default::default()
                    },
                );
            }
        }

        crossovers.sort_by_key(|(kind, pos, dir, res, progress)| {
            // sort vertical before horiz so it draws first
            dir.is_vertical()
        });

        for (kind, pos, dir, res, progress) in crossovers {
            let col = res.color();

            assets
                .shaders
                .cables
                .set_uniform("progress", [col.r, col.g, col.b, progress]);
            assets
                .shaders
                .cables
                .set_uniform("isPipe", if kind == CableKind::Pipe { 1i32 } else { 0 });

            let (cx, cy) = self.board.coord_to_px(pos);
            let ((sx, sy), _) = Cable::Straight {
                horizontal: dir.is_horizontal(),
                kind,
            }
            .get_slices();
            draw_texture_ex(
                assets.textures.cable_atlas,
                cx,
                cy,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(sx, sy + 32.0, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        gl_use_default_material();

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
            let oy = appear_progress.clamp(0.0, 1.0).quad_out(
                patch_height as f32 * -6.0,
                HEIGHT / 2.0 - (16.0 * patch_height as f32) / 2.0 - 48.0,
            ) + if *appear_progress >= 1.0 {
                // sin starts at 0
                ((appear_progress - 1.0) * 0.5).sin() * 5.0
            } else {
                0.0
            };

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
                ox + 6.0,
                oy + 18.0,
                None,
                draw::hexcolor(0xff5277_dd),
                assets,
            );

            gl_use_default_material();
        }

        let text_x = WIDTH / 2.0 - self.level_name.len() as f32 * 4.0 / 2.0;
        draw::pixel_text(
            &self.level_name,
            text_x,
            12.0,
            None,
            hexcolor(0xff5277_ff),
            assets,
        );
    }
}
