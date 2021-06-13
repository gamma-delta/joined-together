use ahash::AHashMap;
use cogs_gamedev::grids::{Direction4, ICoord};
use serde::Deserialize;

use crate::{
    assets::Assets,
    simulator::transport::Resource,
    utils::draw::{self, mouse_position_pixel},
    HEIGHT, WIDTH,
};

use super::transport::{Cable, OmniversalConnector, Port};

/// The board the game is played on.
#[derive(Debug, Clone, Deserialize)]
pub struct Board {
    /// The connector on the spaceship.
    pub left: OmniversalConnector,
    /// The connector on the space station.
    pub right: OmniversalConnector,

    /// The height of the board is determined by the connectors.
    /// This is the width; a width of 7 means X-values from 0-6
    /// can be placed in. (And including the connectors makes it 9, sort of.)
    pub width: usize,
    #[serde(default)]
    pub cables: AHashMap<ICoord, Cable>,
}

impl Board {
    /// Get the port (maybe) at the given position, and the side it is facing.
    pub fn get_port(&self, pos: ICoord) -> Option<(&Port, Direction4)> {
        if pos.y >= 0 {
            let spot = if pos.x == -1 {
                self.left.ports.get(pos.y as usize)
            } else if pos.x == self.width as isize {
                self.right.ports.get(pos.y as usize)
            } else {
                return None;
            };
            spot.map(|x| x.as_ref()).flatten().map(|x| {
                (
                    x,
                    if pos.x == -1 {
                        Direction4::East
                    } else {
                        Direction4::West
                    },
                )
            })
        } else {
            None
        }
    }

    /// Get the height, or the max of the len of the two connectors.
    pub fn height(&self) -> usize {
        self.left.ports.len().max(self.right.ports.len())
    }

    /// Get where the mouse is in ICoords.
    /// The center of this board is centered on the window.
    pub fn mouse_pos(&self) -> ICoord {
        let (mx, my) = mouse_position_pixel();
        let mx = mx - WIDTH / 2.0 - 8.0;
        let my = my - HEIGHT / 2.0 - 8.0;
        let x = (mx / 16.0 + self.width as f32 / 2.0).round() as isize;
        let y = (my / 16.0 + self.height() as f32 / 2.0).round() as isize;
        ICoord::new(x, y)
    }

    /// Get the pixel coordinate of the upper-left coordinate of the ICoord.
    pub fn coord_to_px(&self, pos: ICoord) -> (f32, f32) {
        (
            (pos.x as f32 - self.width as f32 / 2.0) * 16.0 + WIDTH / 2.0,
            (pos.y as f32 - self.height() as f32 / 2.0) as f32 * 16.0 + HEIGHT / 2.0,
        )
    }

    /// Is that position within the cable area?
    pub fn is_in_cable_area(&self, coord: ICoord) -> bool {
        coord.x >= 0
            && coord.x < self.width as isize
            && coord.y >= 0
            && coord.y < self.height() as isize
    }

    /// Is that position within the board OR ports?
    pub fn is_in_board(&self, coord: ICoord) -> bool {
        coord.x >= -1
            && coord.x <= self.width as isize
            && coord.y >= 0
            && coord.y < self.height() as isize
    }

    /// Draw only the stuff on the board (including checkerboard, cables and sides.)
    pub fn draw(&self, assets: &Assets) {
        use macroquad::prelude::*;

        for x in 0..self.width as isize {
            for y in 0..self.height() as isize {
                let pos = ICoord::new(x, y);
                let (cx, cy) = self.coord_to_px(pos);

                let color = if (x + y) % 2 == 0 {
                    // light gray
                    draw::hexcolor(0xa3a7c2_88)
                } else {
                    // dark sea green
                    draw::hexcolor(0x2f5753_60)
                };
                draw_rectangle(cx, cy, 16.0, 16.0, color);
            }
        }

        for (side, left, x) in [
            (&self.left, true, -1),
            (&self.right, false, self.width as isize),
        ] {
            for (y, port) in side.ports.iter().enumerate() {
                if let Some(port) = port {
                    // drawing time
                    let (is_source, res) = match port {
                        Port::Source(res) => (true, res),
                        Port::Sink(res) => (false, res),
                    };

                    let sx = if is_source { 0.0 } else { 16.0 } + if left { 0.0 } else { 32.0 };
                    let (sy, decal) = match res {
                        Resource::Water => (0.0, None),
                        Resource::Fuel => (16.0, None),
                        Resource::Electricity(tw) => {
                            (32.0, Some((3.0, *tw, draw::hexcolor(0xffee83_ff))))
                        }
                        Resource::Data(chan) => {
                            (48.0, Some((8.0, *chan, draw::hexcolor(0xc8d45d_ff))))
                        }
                    };

                    let pos = ICoord::new(x, y as isize);
                    let (cx, cy) = self.coord_to_px(pos);

                    draw_texture_ex(
                        assets.textures.port_atlas,
                        cx,
                        cy,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(sx, sy, 16.0, 16.0)),
                            ..Default::default()
                        },
                    );

                    if let Some((ty, num, color)) = decal {
                        let text = format!("{:02}", num);
                        let tx = if left { 2.0 } else { 7.0 };
                        draw::pixel_text(text, cx + tx, cy + ty, None, color, assets);
                    }
                }
            }
        }

        for (pos, cable) in self.cables.iter() {
            let (cx, cy) = self.coord_to_px(*pos);

            let (sxy1, sxy2) = cable.get_slices();
            for (sx, sy) in std::iter::once(sxy1).chain(sxy2) {
                draw_texture_ex(
                    assets.textures.cable_atlas,
                    cx,
                    cy,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(sx, sy, 16.0, 16.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
