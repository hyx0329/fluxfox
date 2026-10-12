/* Yes, mostly copied directly from RecoveryWhiskers */

use savefile::savefile_derive::Savefile;
use savefile::prelude::Removed;

#[derive(Clone, Copy, Debug, PartialEq, Savefile)]
#[repr(u8)]
pub enum Rotation {
    Partial = 0,
    Full = 1,
}

#[derive(Clone, Debug, Savefile)]
pub struct FluxPart {
    pub data: Vec<u16>,
    pub big_data: Vec<(u64, u32)>,
    pub rotation: Rotation,
    pub freq: f64, // NOTE: not sure if it is still useful

    #[savefile_versions = "..1"]
    pub rotation_time: Removed<f64>,
    #[savefile_versions = "..1"]
    pub phase_period: Removed<f64>,
    #[savefile_versions = "..1"]
    pub phase: Removed<f64>,
    #[savefile_versions = "..1"]
    pub clock: Removed<f64>,
    #[savefile_versions = "..1"]
    pub clock_adj_down: Removed<f64>,
    #[savefile_versions = "..1"]
    pub clock_adj_up: Removed<f64>,
    #[savefile_versions = "..1"]
    pub clock_set: Removed<bool>,

    #[savefile_versions = "1.."]
    pub us: u16,
    #[savefile_versions = "1.."]
    pub xs_max: u16,
    #[savefile_versions = "1.."]
    pub s_max: u16,
    #[savefile_versions = "1.."]
    pub m_max: u16,
    #[savefile_versions = "1.."]
    pub l_max: u16,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub bounds: (u64, u64),
}

impl Default for FluxPart {
    fn default() -> Self {
        Self {
            data: vec![],
            big_data: vec![],
            rotation: Rotation::Full,
            rotation_time: Removed::new(),
            freq: 72_000_000.0,
            phase_period: Removed::new(),
            phase: Removed::new(),
            clock: Removed::new(),
            clock_adj_down: Removed::new(),
            clock_adj_up: Removed::new(),
            clock_set: Removed::new(),
            us: 0,
            xs_max: 0,
            s_max: 0,
            m_max: 0,
            l_max: 0,
            bounds: (0, 0),
        }
    }
}

#[derive(Clone, Debug, Savefile)]
pub struct Flux {
    pub parts: Vec<FluxPart>,
    #[savefile_default_fn = "default_position"]
    #[savefile_versions = "3.."]
    pub position: SideTrack,
}

pub type SideTrack = (u8, u16);

fn default_position() -> SideTrack {
    (0, 0)
}

impl Default for Flux {
    fn default() -> Self {
        Self {
            parts: vec![],
            position: (0, 0),
        }
    }
}
