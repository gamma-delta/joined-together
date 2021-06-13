use cogs_gamedev::grids::{Direction4, Rotation};
use enum_map::{enum_map, EnumMap};
use serde::{Deserialize, Serialize};

/// Anything that can be carried across a cable.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Resource {
    Water,
    Fuel,
    /// This many GW of power.
    /// 0 GW means ground, or something, idk
    Electricity(u8),
    /// Each number represents a specific data line, it must match
    Data(u8),
}

impl Resource {
    /// Get the kind of cable that can carry this
    pub fn appropriate_cable(&self) -> CableKind {
        match self {
            Resource::Water | Resource::Fuel => CableKind::Pipe,
            Resource::Data(_) | Resource::Electricity(_) => CableKind::Wire,
        }
    }
}

/// Mediums of transfer for Resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cable {
    /// The cable goes straight across.
    Straight { kind: CableKind, horizontal: bool },
    /// The cable is bent.
    Bent {
        kind: CableKind,
        /// The direction the counter-clockwise-most end of the cable points to
        ccw_dir: Direction4,
    },
    /// The cable has two cables crossed over
    Crossover {
        horiz_kind: CableKind,
        vert_kind: CableKind,
    },
}

impl Cable {
    /// If a resource enters this cable from the given direction,
    /// where can it exit?
    ///
    /// Returns `Err` if it can't enter here.
    pub fn exit_dir(
        &self,
        resource: &Resource,
        enter_dir: Direction4,
    ) -> Result<Direction4, TransferError> {
        match self {
            Cable::Straight { kind, horizontal } => {
                if !kind.can_carry(&resource) {
                    Err(TransferError::BadCableKind)
                } else if *horizontal == enter_dir.is_horizontal() {
                    // keep going in the direction i came in
                    Ok(enter_dir)
                } else {
                    Err(TransferError::NoEntrance)
                }
            }
            Cable::Bent { kind, ccw_dir } => {
                if !kind.can_carry(&resource) {
                    Err(TransferError::BadCableKind)
                } else {
                    // The other direction this pipe has is:
                    let other_dir = ccw_dir.rotate(Rotation::Clockwise);
                    if enter_dir.flip() == *ccw_dir {
                        Ok(other_dir)
                    } else if enter_dir.flip() == other_dir {
                        Ok(*ccw_dir)
                    } else {
                        Err(TransferError::NoEntrance)
                    }
                }
            }
            Cable::Crossover {
                horiz_kind,
                vert_kind,
            } => {
                let check_kind = if enter_dir.is_horizontal() {
                    horiz_kind
                } else {
                    vert_kind
                };
                if check_kind.can_carry(&resource) {
                    // Nice, keep on trucking
                    Ok(enter_dir)
                } else {
                    Err(TransferError::BadCableKind)
                }
            }
        }
    }

    /// Where can cables point out of this cable?
    pub fn cable_outputs(&self) -> EnumMap<Direction4, Option<CableKind>> {
        match self {
            Cable::Straight { horizontal, kind } => {
                enum_map! {
                    dir => if  dir.is_horizontal() == *horizontal {
                        Some(*kind)
                    } else {None}
                }
            }
            Cable::Bent { ccw_dir, kind } => {
                enum_map! {
                    dir => if *ccw_dir == dir || ccw_dir.rotate(Rotation::Clockwise) == dir {
                        Some(*kind)
                    } else {None}
                }
            }
            Cable::Crossover {
                horiz_kind,
                vert_kind,
            } => {
                enum_map! {
                    dir => Some(if dir.is_horizontal(){*horiz_kind}else{*vert_kind})
                }
            }
        }
    }

    /// Make a cable pointing in the two given directions,
    /// either `Straight` or `Bent`.
    ///
    /// Panics if both are the same direction.
    pub fn from_dirs(kind: CableKind, dir1: Direction4, dir2: Direction4) -> Self {
        if dir1.flip() == dir2 {
            Cable::Straight {
                horizontal: dir1.is_horizontal(),
                kind,
            }
        } else if dir1.rotate(Rotation::Clockwise) == dir2 {
            // dir1 is the CCW direction
            Cable::Bent {
                ccw_dir: dir1,
                kind,
            }
        } else if dir2.rotate(Rotation::Clockwise) == dir1 {
            // dir2 is the CCW one
            Cable::Bent {
                ccw_dir: dir2,
                kind,
            }
        } else {
            panic!("{:?} and {:?} are the same direction", dir1, dir2)
        }
    }

    /// Get the sx/sy of this cable in the atlas.
    /// Also, maybe return a second one to be drawn after if this is a
    /// crossover with two cable types
    pub fn get_slices(&self) -> ((f32, f32), Option<(f32, f32)>) {
        match self {
            Cable::Straight { kind, horizontal } => {
                let sx = if *horizontal { 0.0 } else { 16.0 };
                let sy = match kind {
                    CableKind::Pipe => 0.0,
                    CableKind::Wire => 16.0,
                };
                ((sx, sy), None)
            }
            Cable::Bent { kind, ccw_dir } => {
                // The cables are arranged in Direction4 order in the atlas
                // starting at sx=32.0.
                let sx = 32.0 + (*ccw_dir as u8 as f32) * 16.0;
                let sy = match kind {
                    CableKind::Pipe => 0.0,
                    CableKind::Wire => 16.0,
                };
                ((sx, sy), None)
            }
            Cable::Crossover {
                horiz_kind,
                vert_kind,
            } => {
                if horiz_kind == vert_kind {
                    // Use the special crossover texture
                    let sx = 96.0;
                    let sy = match horiz_kind {
                        CableKind::Pipe => 0.0,
                        CableKind::Wire => 16.0,
                    };
                    ((sx, sy), None)
                } else {
                    // pretend there are two straight cables.
                    // TODO: which looks better on top?
                    let c1 = Cable::Straight {
                        kind: *horiz_kind,
                        horizontal: true,
                    };
                    let c2 = Cable::Straight {
                        kind: *vert_kind,
                        horizontal: false,
                    };
                    let (sxy1, _) = c1.get_slices();
                    let (sxy2, _) = c2.get_slices();
                    (sxy1, Some(sxy2))
                }
            }
        }
    }
}

/// Determines the kind of materials that can go down cables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableKind {
    /// Carries fluids
    Pipe,
    /// Carries electricity
    Wire,
}

impl CableKind {
    /// Can this cable carry the given resource?
    pub fn can_carry(&self, res: &Resource) -> bool {
        match self {
            CableKind::Pipe => matches!(res, Resource::Fuel | Resource::Water),
            CableKind::Wire => matches!(res, Resource::Data(_) | Resource::Electricity(_)),
        }
    }
}

/// Why couldn't the resource be transfered across here?
pub enum TransferError {
    /// The resource didn't line up with the type
    BadCableKind,
    /// We're trying to insert somewhere there isn't an entrance
    NoEntrance,
}

/// The intergalactic "standard" connector.
/// Each level has 2 (or more?) of these.
///
/// The length of the port slots will never change,
/// so please don't push or pop or whatever from the vectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniversalConnector {
    /// Ports, starting at the top.
    /// Index 0 is at y-pos 0, and so on.
    ///
    /// `None` just means there's no port here.
    pub ports: Vec<Option<Port>>,
    /// Positions there are sliders at,
    /// letting you reposition some connectors.
    pub slider: Vec<bool>,
}

/// Different ports in the Omniversal Connectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Port {
    /// It produces this resource
    Source(Resource),
    /// It wants this resource
    Sink(Resource),
}

impl Port {
    pub fn get_resource(&self) -> &Resource {
        match self {
            Port::Source(it) | Port::Sink(it) => it,
        }
    }
}
