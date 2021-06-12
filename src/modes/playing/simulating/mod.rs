mod draw;

use cogs_gamedev::controls::InputHandler;

use crate::{
    assets::Assets,
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    simulator::{
        board::Board,
        floodfill::{FloodFillError, FloodFiller},
        transport::Cable,
    },
};

use super::ModePlaying;

/// Standard time to do one step in frames.
pub(super) const STEP_TIME: u64 = 30;
/// Time to do steps when zooming via tab
const STEP_TIME_ON_DEMAND: u64 = 10;

/// Amount the win box goes in by per frame
const WIN_BOX_ENTER_SPEED: f32 = 2.0 / 30.0;

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
    fn handle_advance(&mut self, controls: &InputSubscriber, frame_info: FrameInfo) -> bool {
        match self.advance_method {
            AdvanceMethod::Errors(..) | AdvanceMethod::WinScreen { .. } => false,
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

    fn step(&mut self) {
        if !self.advance_method.is_special() {
            let errors = self.flooder.step(&self.board);
            if !errors.is_empty() {
                self.advance_method = AdvanceMethod::Errors(errors);
            } else if self.flooder.did_win() {
                // pog
                self.advance_method = AdvanceMethod::WinScreen {
                    text: self.get_win_text(),
                    appear_progress: 0.0,
                }
            }
        }
    }

    fn get_win_text(&self) -> String {
        let chars_across = 20usize;

        // subtract length of "CYCLES:"
        let cycles_metric = format!(
            "CYCLES:{:.>width$}",
            self.flooder.cycles,
            width = chars_across - 7
        );

        let crossovers = self
            .board
            .cables
            .values()
            .filter(|x| matches!(x, Cable::Crossover { .. }))
            .count();
        // length of "CROSSOVERS:"
        let crossover_metric = format!(
            "CROSSOVERS:{:.>width$}",
            crossovers,
            width = chars_across - 11
        );

        format!(
            "{}\n{}\n\n{:^width$}",
            cycles_metric,
            crossover_metric,
            "CLICK ANYWHERE TO CONTINUE",
            width = chars_across
        )
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

        if let AdvanceMethod::WinScreen {
            appear_progress, ..
        } = &mut self.advance_method
        {
            *appear_progress += WIN_BOX_ENTER_SPEED;
            *appear_progress = appear_progress.clamp(0.0, 1.0);
        } else {
            let advance = self.handle_advance(controls, frame_info);
            if advance {
                self.step();
            }
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
    /// Haha (johnathon) we are not actually stepping, instead here's our win screen
    WinScreen {
        /// Progress from 0-1 how in-view our textbox is
        appear_progress: f32,
        /// Text appearing on the win screen
        text: String,
    },
}

impl AdvanceMethod {
    /// Returns `true` if the method is special and won't actually advance
    fn is_special(&self) -> bool {
        matches!(
            self,
            AdvanceMethod::Errors(..) | AdvanceMethod::WinScreen { .. }
        )
    }
}
