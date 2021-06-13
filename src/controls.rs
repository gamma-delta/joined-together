use cogs_gamedev::controls::EventInputHandler;
use enum_map::Enum;
use macroquad::{
    miniquad::{self, Context, KeyMods},
    prelude::{
        utils::{register_input_subscriber, repeat_all_miniquad_input},
        KeyCode, MouseButton,
    },
};

use std::collections::HashMap;

/// The controls
#[derive(Enum, Copy, Clone)]
pub enum Control {
    Select,

    Up,
    Down,
    Left,
    Right,

    Start,
    StepOnce,

    Escape,
    Ctrl,
}

/// Combo keycode and mouse button code
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub enum InputCode {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// Event handler to hook into miniquad and get inputs
#[derive(Clone)]
pub struct InputSubscriber {
    controls: EventInputHandler<InputCode, Control>,
    subscriber_id: usize,
}

impl InputSubscriber {
    pub fn new() -> Self {
        // the science kid
        let sid = register_input_subscriber();

        InputSubscriber {
            controls: EventInputHandler::new(Self::default_controls()),
            subscriber_id: sid,
        }
    }

    pub fn default_controls() -> HashMap<InputCode, Control> {
        let mut controls = HashMap::new();

        // Put your controls here
        for (code, control) in [
            (KeyCode::Enter, Control::Select),
            //
            (KeyCode::W, Control::Up),
            (KeyCode::Up, Control::Up),
            (KeyCode::A, Control::Left),
            (KeyCode::Left, Control::Left),
            (KeyCode::S, Control::Down),
            (KeyCode::Down, Control::Down),
            (KeyCode::D, Control::Right),
            (KeyCode::Right, Control::Right),
            //
            (KeyCode::Space, Control::Start),
            (KeyCode::Tab, Control::StepOnce),
            //
            (KeyCode::Escape, Control::Escape),
            (KeyCode::LeftControl, Control::Ctrl),
            (KeyCode::RightControl, Control::Ctrl),
        ] {
            controls.insert(InputCode::Key(code), control);
        }
        controls.insert(InputCode::Mouse(MouseButton::Left), Control::Select);

        controls
    }

    pub fn update(&mut self) {
        repeat_all_miniquad_input(self, self.subscriber_id);
        self.controls.update();
    }
}

impl std::ops::Deref for InputSubscriber {
    type Target = EventInputHandler<InputCode, Control>;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}

impl miniquad::EventHandler for InputSubscriber {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, _ctx: &mut Context) {}

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            self.controls.input_down(InputCode::Key(keycode));
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.controls.input_up(InputCode::Key(keycode));
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.controls.input_down(InputCode::Mouse(button));
    }
    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        self.controls.input_up(InputCode::Mouse(button));
    }
}
