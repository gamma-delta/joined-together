use ahash::AHashMap;
use cogs_gamedev::grids::{Direction4, ICoord};

use crate::simulator::transport::TransferError;

use super::{
    board::Board,
    transport::{Port, Resource},
};

/// This lets us do a floodfill over several frames.
#[derive(Clone, Debug)]
pub struct FloodFiller {
    /// Active "tips", or frontiers we're moving resources at.
    /// Becomes None when the tip is satisfied at a Sink.
    pub tips: Vec<Option<Tip>>,
    /// Spaces we've already visited. (This should never overlap with any Tip.)
    /// The boolean is for horizontality; were we horizontal when we were in this space?
    ///
    /// This is purely for drawing purposes and NOT for the flood-fill itself!
    pub visited: AHashMap<(ICoord, bool), Resource>,
    pub cycles: u64,
}

#[derive(Clone, Debug)]
pub struct Tip {
    /// Current position
    pub pos: ICoord,
    /// Direction this entered the current coordinate from.
    pub facing: Direction4,
    /// The resource this is carrying
    pub resource: Resource,
}

impl FloodFiller {
    /// Make a new FloodFiller operating on the given board.
    pub fn new(board: &Board) -> Self {
        let mut tips = Vec::new();
        for (conn, dir, x) in [
            // Ports on the left push their stuff east at column 0
            (&board.left, Direction4::East, 0),
            // Ports on the right push their stuff west at column (width-1)
            (&board.right, Direction4::West, board.width - 1),
        ] {
            for (y, port) in conn.ports.iter().enumerate() {
                if let Some(Port::Source(res)) = port {
                    tips.push(Some(Tip {
                        pos: ICoord::new(x as isize, y as isize),
                        facing: dir,
                        resource: res.clone(),
                    }));
                }
            }
        }

        Self {
            tips,
            visited: AHashMap::new(),
            cycles: 0,
        }
    }

    /// Do one flood-fill step.
    ///
    /// If any problems happened we return them in the vector.
    /// If it's empty, we're all set!
    pub fn step(&mut self, board: &Board) -> Vec<FloodFillError> {
        self.cycles += 1;

        let mut errors = Vec::new();

        // clippy is overzealous here
        #[allow(clippy::manual_flatten)]
        for tip_slot in self.tips.iter_mut() {
            if let Some(tip) = tip_slot {
                if self
                    .visited
                    .insert((tip.pos, tip.facing.is_horizontal()), tip.resource.clone())
                    .is_some()
                {
                    errors.push(FloodFillError::Backtrack(tip.pos));
                    continue;
                }

                if let Some(current_cable) = board.cables.get(&tip.pos) {
                    let out_dir = match current_cable.exit_dir(&tip.resource, tip.facing) {
                        Ok(it) => it,
                        Err(ono) => {
                            let err = match ono {
                                TransferError::BadCableKind => {
                                    FloodFillError::BadCableKind(tip.pos)
                                }
                                TransferError::NoEntrance => FloodFillError::NoEntrance(tip.pos),
                            };
                            errors.push(err);
                            continue;
                        }
                    };
                    let target_pos = tip.pos + out_dir;
                    if let Some(target_cable) = board.cables.get(&target_pos) {
                        tip.pos = target_pos;
                        tip.facing = out_dir;
                    } else {
                        // Perhaps we are "spilling" into an exit.
                        if let Some((Port::Sink(res), _)) = board.get_port(target_pos) {
                            if res != &tip.resource {
                                // oh no...
                                errors.push(FloodFillError::BadOutput(target_pos, res.clone()))
                            } else {
                                // we are done here poggers
                                *tip_slot = None;
                            }
                        } else {
                            // Nope we spill into space
                            errors.push(FloodFillError::SpilledIntoSpace(target_pos));
                        }
                    }
                } else {
                    // Really don't know how we got here but uh
                    errors.push(FloodFillError::SpilledIntoSpace(tip.pos));
                }
            }
        }

        errors
    }

    /// Did we win?
    pub fn did_win(&self) -> bool {
        self.tips.iter().all(Option::is_none)
    }
}

#[derive(Clone)]
pub enum FloodFillError {
    BadCableKind(ICoord),
    NoEntrance(ICoord),
    /// We spilled something into the vacuum of space.
    SpilledIntoSpace(ICoord),
    /// We somehow tried to go back along a pipe we previously went on
    Backtrack(ICoord),
    /// The port didn't like the resource given
    BadOutput(ICoord, Resource),
}
