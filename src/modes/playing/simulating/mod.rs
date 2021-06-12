mod draw;

use cogs_gamedev::controls::InputHandler;

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    simulator::{
        board::Board,
        floodfill::{FloodFillError, FloodFiller},
    },
};

use super::ModePlaying;

/// Standard time to do one step in frames.
pub(super) const STEP_TIME: u64 = 30;
/// Time to do steps when zooming via tab
const STEP_TIME_ON_DEMAND: u64 = 10;

#[derive(Clone)]
pub(super) struct ModeSimulating {
    /// We do need to clone into this, which is kind of bad,
    /// but it makes things simpler and i really doubt there's
    /// going to be bad perf issues from cloning like 1kb
    board: Board,
    flooder: FloodFiller,

    advance_method: AdvanceMethod,
}

impl ModeSimulating {
    pub fn new(mode: &ModePlaying, advance_method: AdvanceMethod) -> Self {
        Self {
            board: mode.board.clone(),
            flooder: FloodFiller::new(&mode.board),
            advance_method,
        }
    }

    /// Update my advancing method.
    /// Return if we should step.
    pub fn handle_advance(&mut self, controls: &InputSubscriber, frame_info: FrameInfo) -> bool {
        match self.advance_method {
            AdvanceMethod::Errors(..) => false,
            _ if controls.clicked_down(Control::StepOnce) => {
                self.advance_method = AdvanceMethod::OnDemand;
                true
            }
            AdvanceMethod::ByFrames {
                start_frame,
                interval,
            } => {
                if controls.clicked_down(Control::Start) {
                    // pause
                    self.advance_method = AdvanceMethod::OnDemand;
                    false
                } else {
                    // Check if we're on a hot frame
                    let dframe = frame_info.frames_ran - start_frame;
                    dframe % interval == 0
                }
            }
            AdvanceMethod::OnDemand => {
                if controls.clicked_down(Control::Start) {
                    // back to automatic play
                    self.advance_method = AdvanceMethod::ByFrames {
                        start_frame: frame_info.frames_ran,
                        interval: STEP_TIME,
                    };
                    true
                } else {
                    // just waiting
                    false
                }
            }
        }
    }

    pub fn step(&mut self) {
        if !self.advance_method.is_errors() {
            let errors = self.flooder.step(&self.board);
            if !errors.is_empty() {
                self.advance_method = AdvanceMethod::Errors(errors);
            }
        }
    }
}

impl Gamemode for ModeSimulating {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Escape) {
            return Transition::Pop;
        }

        let advance = self.handle_advance(controls, frame_info);
        if advance {
            self.step();
        }

        Transition::None
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub(super) enum AdvanceMethod {
    /// Advance the flood fill once every this many frames
    ByFrames { start_frame: u64, interval: u64 },
    /// Advance it on demand when tab is pressed
    OnDemand,
    /// Wait there were errors!
    Errors(Vec<FloodFillError>),
}

impl AdvanceMethod {
    /// Returns `true` if the advance_method is [`Errors`].
    fn is_errors(&self) -> bool {
        matches!(self, Self::Errors(..))
    }
}
