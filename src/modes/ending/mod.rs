use cogs_gamedev::controls::InputHandler;
use macroquad::prelude::{draw_text, gl_use_default_material, gl_use_material};

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, RenderTargetStack, Transition},
    controls::{Control, InputSubscriber},
    utils::draw::{self, draw_space},
    HEIGHT, WIDTH,
};

const TEXTBOX_WIDTH: usize = 11;
const TEXTBOX_HEIGHT: usize = 11;

const CORNER_X: f32 = WIDTH / 2.0 - TEXTBOX_WIDTH as f32 * 16.0 / 2.0;
const CORNER_Y: f32 = HEIGHT / 2.0 - TEXTBOX_HEIGHT as f32 * 16.0 / 2.0;

const TEXT_OFFSET_X: f32 = 10.0;
const TEXT_OFFSET_Y: f32 = 12.0;

#[derive(Clone)]
pub struct ModeEnding {}

impl ModeEnding {
    pub fn new() -> Self {
        Self {}
    }
}

impl Gamemode for ModeEnding {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Escape) {
            Transition::Pop
        } else {
            Transition::None
        }
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }
}

impl GamemodeDrawer for ModeEnding {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo, render_targets: &mut RenderTargetStack) {
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

        let text = r#"
YOU BEAT THE GAME! CONGRATULATIONS!

IF YOU LIKED THE GAME, WHY NOT LEAVE A
COMMENT AND A NICE REVIEW? 
I REALLY APPRECIATE IT.


CREDITS:
- GAMMA-DELTA - CODING, DESIGN, ART,
  PRETTY MUCH EVERYTHING ELSE
- ALWINFY - CODE CLEANUP (MAYBE, 
  DEPENDING ON IF THEY EVER PUSH 
  ANYTHING, AHEM)
- ZACH BARTH - MAKING SPACECHEM, 
  WHICH THIS GAME WAS INSPIRED BY
- AND YOU! THANK YOU FOR PLAYING!


               -=: FIN :=-




                  [PRESS ESC TO GO BACK]
        "#;
        draw::pixel_text(
            text,
            CORNER_X + TEXT_OFFSET_X,
            CORNER_Y + TEXT_OFFSET_Y,
            None,
            draw::hexcolor(0xff5277_dd),
            assets,
        );

        gl_use_default_material();
    }
}
