use cogs_gamedev::grids::ICoord;

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, GamemodeDrawer, RenderTargetStack},
    modes::playing::draw_space,
    simulator::board::Board,
    utils::draw,
};

use super::ModePlaying;

pub(super) struct Drawer {
    /// Board, possibly ersatz and constructed piecewise out of various places.
    board: Board,
    /// Hover position of the cursor
    cursor: ICoord,
    /// If this is `true` the player is selecting at the cursor
    selecting: bool,

    start_time: f64,
}

impl Drawer {
    pub fn new(mode: &ModePlaying) -> Self {
        let cables = if let Some(sel) = &mode.selection {
            sel.cables.clone()
        } else {
            mode.board.cables.clone()
        };
        let board = Board {
            cables,
            left: mode.board.left.clone(),
            right: mode.board.right.clone(),
            width: mode.board.width,
        };
        Self {
            board,
            cursor: mode.cursor,
            selecting: mode.selection.is_some(),
            start_time: mode.start_time,
        }
    }
}

impl GamemodeDrawer for Drawer {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
        use macroquad::prelude::*;

        draw_space(assets, frame_info.frames_ran as f32 / 30.0);

        self.board.draw(assets);

        let dt = macroquad::time::get_time() - self.start_time;
        let mut cursor_color = if self.selecting {
            // sea green
            draw::hexcolor(0x92e8c0_ff)
        } else {
            // yellow
            draw::hexcolor(0xffee83_dd)
        };
        cursor_color.a = 0.6 - ((dt * 3.0).sin() as f32 + 1.0 / 2.0) * 0.1;
        let (cx, cy) = self.board.coord_to_px(self.cursor);
        draw_rectangle(cx, cy, 16.0, 16.0, cursor_color);
    }
}
