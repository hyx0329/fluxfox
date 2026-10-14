/* Yes, mostly copied directly from RecoveryWhiskers */

use savefile::savefile_derive::Savefile;

use super::flux::*;
use std::{
    collections::BTreeMap,
};

pub(super) const RWR_VER: u32 = 4;
// new type specifies u32
type FluxLookupRWR = BTreeMap<(u8, u16), Vec<u32>>;

// old types, note here we assume old usize is actually u64, that means files generated on 32bit platforms are not supported
type FluxLookupRWR1 = BTreeMap<(u8, u8), Vec<u64>>;
type FluxLookupRWR2_3 = BTreeMap<(u8, u16), Vec<u64>>;

/* NOTE on converting to other formats:
 *  flux_lookup has lookups for side, track that is either parsed (if successful) or the physical location. the fact it is parsed has no guarantee that all of the sectors parsed are on that same track. Since RWR version 3, Flux has a position that is the physical position of the drive. If you want to convert to a raw format, you should probably use that.
 *  The physical position has is stored as is, it has no logic about translating itself to the logical sector (i.e. if you have a 40 track floppy in an 80 track drive, you will get the tracks corresponding to the 80 track drive here)
 *  RWR version 4 standardizes the index lookup to u32. For converting older files, the files must be opened with the same pointer size. It is assumed that files won't have larger lookups than u32::MAX
 *
 * */

#[derive(Clone, Debug, Savefile)]
pub struct RecoveryWhiskersRaw {
    pub fluxes: Vec<Flux>,
    #[savefile_ignore]
    #[savefile_introspect_ignore]
    pub changed: bool,
    #[savefile_versions_as = "1..1:convert_rwr_1_btreemap:FluxLookupRWR1"]
    #[savefile_versions_as = "2..3:convert_rwr_usize_btreemap:FluxLookupRWR2_3"]
    #[savefile_versions = "4.."]
    pub flux_lookup: FluxLookupRWR,
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

fn convert_rwr_1_btreemap(x: FluxLookupRWR1) -> FluxLookupRWR {
    let mut res = BTreeMap::new();
    for ((side, track), old_vec) in x.iter() {
        let mut new_vec = vec![];
        for value in old_vec.iter() {
            new_vec.push(u32::try_from(*value).unwrap_or(u32::MAX));
        }
        res.insert((*side, *track as u16), new_vec);
    }
    res
}

/// converts RWR versions 2 and 3 to Version 4 which has its flux_lookup index as u32
fn convert_rwr_usize_btreemap(x: FluxLookupRWR2_3) -> FluxLookupRWR {
    let mut res = BTreeMap::new();
    for ((side, track), old_vec) in x.iter() {
        let mut new_vec = vec![];
        for value in old_vec.iter() {
            new_vec.push(u32::try_from(*value).unwrap_or(u32::MAX));
        }

        res.insert((*side, *track), new_vec);
    }
    res
}
