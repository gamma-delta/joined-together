mod drawer;
mod simulating;

use ahash::AHashMap;
use cogs_gamedev::{
    controls::InputHandler,
    grids::{Direction4, ICoord},
};
use macroquad::prelude::info;

use crate::{
    assets::{Assets, Level},
    boilerplates::{FrameInfo, Gamemode, GamemodeDrawer, Transition},
    controls::{Control, InputSubscriber},
    simulator::{
        board::Board,
        solutions::Solution,
        transport::{Cable, OmniversalConnector, Port, Resource},
    },
    utils::{draw::draw_space, profile::Profile},
    HEIGHT, WIDTH,
};

use self::{
    drawer::Drawer,
    simulating::{AdvanceMethod, ModeSimulating, STEP_TIME},
};

pub struct ModePlaying {
    board: Board,

    /// Where the cursor is hovering
    cursor: ICoord,
    selection: Option<Selection>,

    start_time: f64,

    level_key: String,
    level_idx: usize,
    level_name: String,
}

/// Info about dragging pipes around.
pub struct Selection {
    /// Previous selection positions and the direction we moved in to get there,
    /// or None if this is brand new.
    prev_info: Vec<(ICoord, Direction4)>,

    /// Current cable state. When we're selecting and editing pipes,
    /// we update this and then clobber the board with this
    /// once we finish.
    cables: AHashMap<ICoord, Cable>,
}

impl ModePlaying {
    pub fn new(level: &Level, level_idx: usize) -> Self {
        let mut profile = Profile::get();
        let solution = profile
            .solutions
            .entry(level.filename.clone())
            .or_insert_with(|| Solution {
                cables: level.starting_board.cables.clone(),
                left: level.starting_board.left.clone(),
                right: level.starting_board.right.clone(),
                metrics: None,
                level_key: level.filename.clone(),
            })
            .clone();
        let board = Board {
            cables: solution.cables,
            left: solution.left,
            right: solution.right,
            width: level.starting_board.width,
        };
        let cursor = ICoord::new(board.width as isize / 2, board.height() as isize / 2);

        ModePlaying {
            board,
            cursor,
            selection: None,
            start_time: macroquad::time::get_time(),
            level_key: level.filename.clone(),
            level_idx,
            level_name: level.name.clone(),
        }
    }

    fn handle_selection(&mut self, controls: &InputSubscriber) {
        match &mut self.selection {
            None => {
                let maybe_cursor = self.board.mouse_pos();
                if controls.clicked_down(Control::Select) {
                    info!("Clicked at {:?}", maybe_cursor);
                }
                if self.board.is_in_board(maybe_cursor) {
                    self.cursor = maybe_cursor;

                    if controls.clicked_down(Control::Select) {
                        // Check if there's a cable or port here
                        if self.board.cables.contains_key(&self.cursor)
                            || self.board.get_port(self.cursor).is_some()
                        {
                            self.selection = Some(Selection {
                                prev_info: Vec::new(),
                                cables: self.board.cables.clone(),
                            })
                        }
                    }
                }
            }
            Some(selection) => {
                let mut save_current = false;

                if controls.clicked_down(Control::Select) {
                    // ok we successfully ended!
                    save_current = true;
                } else {
                    // ok let's try to keep dragging things around
                    let maybe_cursor = self.board.mouse_pos();
                    // How far did the cursor move?
                    let dcursor = maybe_cursor + ICoord::new(-self.cursor.x, -self.cursor.y);
                    let dir = match (dcursor.x, dcursor.y) {
                        (1, 0) => Some(Direction4::East),
                        (-1, 0) => Some(Direction4::West),
                        (0, 1) => Some(Direction4::South),
                        (0, -1) => Some(Direction4::North),
                        _ => None,
                    };
                    if let Some(dir) = dir {
                        // We moved by an OK direction
                        let current_cable = selection.cables.get(&self.cursor);

                        #[derive(PartialEq, Eq)]
                        enum Continue {
                            KeepAdding,
                            DontAdd,
                            MergeSelection,
                        }

                        let continue_adding = if let Some(current_cable) = current_cable {
                            // If we are backtracking along the cable, actually
                            // remove the current cable instead.
                            let last_push = selection.prev_info.last();
                            let continue_ = if let Some((last_pos, _)) = last_push {
                                if maybe_cursor == *last_pos {
                                    // backtrack pog
                                    // are we backtracking into a port?
                                    if self.board.get_port(*last_pos).is_some() {
                                        Continue::MergeSelection
                                    } else {
                                        Continue::DontAdd
                                    }
                                } else {
                                    Continue::KeepAdding
                                }
                            } else {
                                // Perhaps we are newly picking up a cable.
                                // Check if there is at least one unoccupied exit and we are not heading towards it.

                                // We want to backtrack in this special case if:
                                // - Our current has a cable
                                // - Target has either a cable or port
                                // - Current and target are connected
                                // - Current has at least one unoccupied exit (where ports count as free)
                                let current_outputs = current_cable.cable_outputs();
                                let current_target_connect = if let Some(target_cable) =
                                    selection.cables.get(&maybe_cursor)
                                {
                                    let target_outputs = target_cable.cable_outputs();
                                    current_outputs[dir].is_some()
                                        && target_outputs[dir.flip()].is_some()
                                } else if let Some((_, port_dir)) =
                                    self.board.get_port(maybe_cursor)
                                {
                                    port_dir == dir.flip()
                                } else {
                                    false
                                };

                                // We count ports as free so we can disconnect things from them
                                let fully_occupied = is_fully_occupied(
                                    self.cursor,
                                    &selection.cables,
                                    &self.board,
                                    false,
                                    false,
                                );

                                if current_target_connect && (fully_occupied != Some(true)) {
                                    info!("Special backtrack, deleting {:?}", self.cursor);
                                    Continue::DontAdd
                                } else {
                                    Continue::KeepAdding
                                }
                            };

                            if continue_ != Continue::KeepAdding {
                                // ok we backtracked!
                                if let Cable::Crossover {
                                    horiz_kind,
                                    vert_kind,
                                } = current_cable
                                {
                                    // turn this into a singlet cable
                                    // invert the horizontal-ness;
                                    // if the backtrack direction is horizontal, leave vertical
                                    let horizontal = !dir.is_horizontal();
                                    let new_cable = Cable::Straight {
                                        horizontal,
                                        kind: if horizontal { *horiz_kind } else { *vert_kind },
                                    };
                                    selection.cables.insert(self.cursor, new_cable);
                                } else {
                                    // just remove it
                                    selection.cables.remove(&self.cursor);
                                }
                            }
                            continue_
                        } else {
                            Continue::KeepAdding
                        };

                        if continue_adding == Continue::KeepAdding {
                            // Here's the cable or port we're currently operating on...
                            // re-get the values because of mutable borrows, even though
                            // I *know* it's ok due to the bools...
                            let current_cable = selection.cables.get(&self.cursor);
                            let current_port = self.board.get_port(self.cursor);
                            if current_cable.is_some() || current_port.is_some() {
                                // If we are exiting from a crossover cable, we need to make sure
                                // we're not turning.
                                // Also, we need to make sure we're not backtracking.
                                let ok_dir = match (current_cable, selection.prev_info.last()) {
                                    // Only allow the *same* direction we started in for crossovers
                                    (Some(Cable::Crossover { .. }), Some((_, prev_dir)))
                                        if *prev_dir == dir =>
                                    {
                                        true
                                    }
                                    // Allow any direction as long as its not the entering dir.
                                    (Some(_), Some((_, prev_dir))) => prev_dir.flip() != dir,
                                    // Allow anything if you're just starting out
                                    (_, None) => true,
                                    // Only allow outwards from a port
                                    (None, _) if current_port.is_some() => {
                                        if self.cursor.x == -1 {
                                            dir == Direction4::East
                                        } else {
                                            dir == Direction4::West
                                        }
                                    }
                                    _ => unreachable!(
                                        "At least one should be Some: {:?}, {:?}",
                                        current_cable, current_port
                                    ),
                                };
                                // Also, no editing something that is entirely occupied
                                let fully_occupied = is_fully_occupied(
                                    self.cursor,
                                    &selection.cables,
                                    &self.board,
                                    true,
                                    true,
                                );
                                // Also, no going out of bounds or onto something that isn't a port
                                let in_bounds = self.board.is_in_cable_area(maybe_cursor)
                                    || self.board.get_port(maybe_cursor).is_some();

                                if ok_dir && (fully_occupied != Some(true)) && in_bounds {
                                    // Alright we're cleared to place our new cable,
                                    // and maybe edit this one.
                                    let current_kind = match current_cable {
                                        Some(
                                            Cable::Straight { kind, .. } | Cable::Bent { kind, .. },
                                        ) => *kind,
                                        Some(Cable::Crossover {
                                            horiz_kind,
                                            vert_kind,
                                        }) => {
                                            if dir.is_horizontal() {
                                                *horiz_kind
                                            } else {
                                                *vert_kind
                                            }
                                        }
                                        None => {
                                            if let Some((port, _)) = current_port {
                                                port.get_resource().appropriate_cable()
                                            } else {
                                                // should have checked this already
                                                unreachable!()
                                            }
                                        }
                                    };

                                    // Pre-calculate this cable but only insert it if everything goes well
                                    // because we may need to bend it.
                                    let new_current_cable = if let Some(Cable::Crossover {
                                        ..
                                    }) = current_cable
                                    {
                                        // Don't un-crossover it.
                                        current_cable.cloned()
                                    } else if let Some((_, prev_dir)) = selection.prev_info.last() {
                                        // We *exited* via `prev_dir`, so we enter via it flipped
                                        Some(Cable::from_dirs(current_kind, dir, prev_dir.flip()))
                                    } else if let Some(current_cable) = current_cable {
                                        // We don't have a backup to rely on, so we have to figure out
                                        // where we used to be facing.
                                        // The "stable" direction is a direction:
                                        // - that's connected to something
                                        // - that isn't the direction we're exiting in
                                        let cursor = self.cursor;
                                        let cables = &selection.cables;
                                        let board = &self.board;
                                        let stable = current_cable.cable_outputs().iter().find_map(
                                            |(look_dir, kind)| {
                                                if look_dir == dir {
                                                    return None;
                                                }
                                                if let Some(kind) = *kind {
                                                    // Check cables and ports in this direction
                                                    let target_pos = cursor + look_dir;
                                                    if let Some(neighbor) = cables.get(&target_pos)
                                                    {
                                                        let neighbor_conn =
                                                            neighbor.cable_outputs()[dir.flip()];
                                                        // if this is some, we have two cables facing each other
                                                        if neighbor_conn.is_some() {
                                                            // we're wired up!
                                                            Some((look_dir, kind))
                                                        } else {
                                                            None
                                                        }
                                                    } else if let Some((_, port_dir)) =
                                                        board.get_port(target_pos)
                                                    {
                                                        // if these are the same, that means these face each other so it's occupied
                                                        if port_dir == dir.flip() {
                                                            Some((look_dir, kind))
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        // this direction is pointing to nothing
                                                        None
                                                    }
                                                } else {
                                                    // Not connected to anything in this dir
                                                    None
                                                }
                                            },
                                        );
                                        stable.map(|(stable_dir, kind)| {
                                            Cable::from_dirs(kind, dir, stable_dir)
                                        })
                                    } else {
                                        // This handily works out if current_cable is None
                                        // cause that means we don't want to put something over our port
                                        None
                                    };
                                    // Now, either insert a new cable,
                                    // or update an existing cable to be a crossover.
                                    let (success, end) =
                                        match selection.cables.get_mut(&maybe_cursor) {
                                            None => {
                                                // We are not clobbering anything! Nice!
                                                if self.board.get_port(maybe_cursor).is_none() {
                                                    // Don't try to put a pipe on top of a port
                                                    selection.cables.insert(
                                                        maybe_cursor,
                                                        Cable::Straight {
                                                            horizontal: dir.is_horizontal(),
                                                            kind: current_kind,
                                                        },
                                                    );
                                                    (true, false)
                                                } else {
                                                    // we are mousing over a port! time to end
                                                    (true, true)
                                                }
                                            }
                                            Some(Cable::Straight {
                                                horizontal: target_horizontal,
                                                kind: target_kind,
                                            }) => {
                                                // OK we need to make sure we are *crossing* it.
                                                if *target_horizontal != dir.is_horizontal() {
                                                    // nice let's insert our crossover
                                                    let (h, v) = if *target_horizontal {
                                                        (*target_kind, current_kind)
                                                    } else {
                                                        (current_kind, *target_kind)
                                                    };
                                                    selection.cables.insert(
                                                        maybe_cursor,
                                                        Cable::Crossover {
                                                            horiz_kind: h,
                                                            vert_kind: v,
                                                        },
                                                    );
                                                    (true, false)
                                                } else {
                                                    (false, false)
                                                }
                                            }
                                            // trying to clobber something we can't turn into a crossover
                                            _ => (false, false),
                                        };
                                    if success {
                                        // hooray
                                        if let Some(new_current_cable) = new_current_cable {
                                            selection.cables.insert(self.cursor, new_current_cable);
                                        }
                                        selection.prev_info.push((self.cursor, dir));

                                        if end {
                                            // time to quit
                                            save_current = true;
                                        }

                                        self.cursor = maybe_cursor;
                                    }
                                } else {
                                    info!("Failed to place new cable; ok_dir: {}; fully_occupied: {:?}", ok_dir, fully_occupied);
                                }
                            } else {
                                // uh oh I don't know how we got here but it sure isn't valid
                                // this means we're not selecting a cable nor a port?
                                self.selection = None;
                            }
                        } else if continue_adding == Continue::MergeSelection {
                            // we backtracked all the way to a port!
                            save_current = true;
                        } else {
                            // We backtracked just once
                            selection.prev_info.pop();
                            self.cursor = maybe_cursor;
                        }
                    } // Else we moved too fast, just keep it the same...
                }

                if save_current {
                    let sel = self.selection.take().unwrap();
                    self.board.cables = sel.cables;

                    let mut profile = Profile::get();
                    profile.solutions.insert(
                        self.level_key.clone(),
                        Solution {
                            level_key: self.level_key.clone(),
                            cables: self.board.cables.clone(),
                            left: self.board.left.clone(),
                            right: self.board.right.clone(),
                            metrics: None,
                        },
                    );
                }
            }
        }
    }
}

impl Gamemode for ModePlaying {
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition {
        if controls.clicked_down(Control::Escape) {
            return Transition::Pop;
        }

        let method = if controls.clicked_down(Control::StepOnce) {
            Some(AdvanceMethod::OnDemand)
        } else if controls.clicked_down(Control::Start) {
            Some(AdvanceMethod::ByFrames {
                start_frame: frame_info.frames_ran,
                interval: STEP_TIME,
            })
        } else {
            None
        };
        if let Some(method) = method {
            return Transition::Push(Box::new(ModeSimulating::new(
                &self,
                method,
                frame_info.frames_ran,
            )));
        }

        self.handle_selection(controls);

        Transition::None
    }

    fn get_draw_info(&mut self) -> Box<dyn GamemodeDrawer> {
        Box::new(Drawer::new(&self))
    }
}

/// Does the cable at the given position have all of its exits used?
/// Returns `None` if there is no cable there
///
/// `port_board` is used only for the ports.
///
/// If `count_ports` is false, ports count as open space.
///
/// If `allow_bend` is true, it will check if there's any open space
/// next to directions it doesn't have a cable too.
fn is_fully_occupied(
    pos: ICoord,
    cables: &AHashMap<ICoord, Cable>,
    port_board: &Board,
    count_ports: bool,
    allow_bend: bool,
) -> Option<bool> {
    cables.get(&pos).map(|cable| {
        // Check if none (not any) of the sides are free.
        !cable.cable_outputs().iter().any(|(dir, kind)| {
            let target_pos = pos + dir;
            if kind.is_some() {
                if let Some(neighbor) = cables.get(&target_pos) {
                    let neighbor_conn = neighbor.cable_outputs()[dir.flip()];
                    // if this is some, we have two cables facing each other
                    neighbor_conn.is_none()
                } else if let (true, Some((_, port_dir))) =
                    (count_ports, port_board.get_port(target_pos))
                {
                    // if these are the same, that means these face each other so it's occupied
                    port_dir != dir.flip()
                } else {
                    // this direction is pointing to nothing! and free!
                    true
                }
            } else {
                // There's no output here so it is not free
                allow_bend
            }
        })
    })
}
