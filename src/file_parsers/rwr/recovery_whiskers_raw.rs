/* Yes, mostly copied directly from RecoveryWhiskers
 * Critical note: DO CHECK USIZE, as most wasm targets are 32bits.
 */

use savefile::savefile_derive::Savefile;

use super::flux::*;
use log::*;
use std::{
    collections::BTreeMap,
    fs,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

const RWR_VER: u32 = 3;
type FluxLookupRWR1 = BTreeMap<(u8, u8), Vec<u64>>;

/* NOTE on converting to other formats:
 *  flux_lookup has lookups for side, track that is either parsed (if successful) or the physical location. the fact it is parsed has no guarantee that all of the sectors parsed are on that same track. Since RWR version 3, Flux has a position that is the physical position of the drive. If you want to convert to a raw format, you should probably use that.
 *  The physical position has is stored as is, it has no logic about translating itself to the logical sector (i.e. if you have a 40 track floppy in an 80 track drive, you will get the tracks corresponding to the 80 track drive here)
 *
 *
 * */

#[derive(Clone, Debug, Savefile)]
pub struct RecoveryWhiskersRaw {
    pub fluxes: Vec<Flux>,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub changed: bool,
    #[savefile_versions_as = "1..1:convert_rwr_1_btreemap:FluxLookupRWR1"]
    #[savefile_versions = "2.."]
    pub flux_lookup: BTreeMap<(u8, u16), Vec<u64>>,
}

impl Default for RecoveryWhiskersRaw {
    fn default() -> Self {
        Self {
            fluxes: vec![],
            changed: false,
            flux_lookup: BTreeMap::new(),
        }
    }
}

fn convert_rwr_1_btreemap(x: FluxLookupRWR1) -> BTreeMap<(u8, u16), Vec<u64>> {
    let mut res = BTreeMap::new();
    for ((side, track), val) in x.iter() {
        res.insert((*side, *track as u16), val.clone());
    }
    res
}
