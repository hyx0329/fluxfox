/*
    Sorry but no copyright notice yet!
*/

mod recovery_whiskers_raw;
mod flux;

use crate::{
    DiskImage, DiskImageError, DiskImageFileFormat, LoadingCallback, ParserWriteCompatibility, file_parsers::{FormatCaps, ParserReadOptions, ParserWriteOptions, bitstream_flags}, io::{ReadSeek, ReadWriteSeek}, track::fluxstream::FluxStreamTrack, types::{DiskCh, DiskDescriptor, Platform, TrackDataEncoding, TrackDataResolution, TrackDensity}
};

use crate::types::FluxStreamTrackParams;
use strum::IntoEnumIterator;

use flux::{FluxPart, Rotation};
use recovery_whiskers_raw::RecoveryWhiskersRaw;
use savefile::load;
use std::collections::BTreeMap;

// Not exported by recovery_whiskers
const RWR_VER: u32 = 3;

pub struct RwrFormat;

impl RwrFormat {
    pub(crate) fn extensions() -> Vec<&'static str> {
        vec!["rwr"]
    }

    pub(crate) fn capabilities() -> FormatCaps {
        bitstream_flags()
    }

    pub(crate) fn platforms() -> Vec<Platform> {
        // Theoretically RWR should support everything
        Platform::iter().collect()
    }

    pub(crate) fn detect<RWS: ReadSeek>(mut image: RWS) -> bool {
        // theres no known way to know if it's valid without loading it all
        if image.seek(std::io::SeekFrom::Start(0)).is_err() {
            return false;
        }

        let mut buffer = [0u8; 8];
        let _ = image.read(&mut buffer);
        if buffer != "savefile".as_bytes() {
            return false;
        }

        let _ = image.seek(std::io::SeekFrom::Start(0));
        load::<RecoveryWhiskersRaw>(&mut image, RWR_VER).is_ok()
    }

    pub fn can_write(_image: Option<&DiskImage>) -> ParserWriteCompatibility {
        // Technically can write but it just makes no sense
        ParserWriteCompatibility::UnsupportedFormat
    }

    pub(crate) fn load_image<RWS: ReadSeek>(
        mut image: RWS,
        disk_image: &mut DiskImage,
        _opts: &ParserReadOptions,
        _callback: Option<LoadingCallback>,
    ) -> Result<(), DiskImageError> {
        disk_image.set_resolution(TrackDataResolution::FluxStream);
        disk_image.set_source_format(DiskImageFileFormat::RecoveryWhiskersRaw);

        if image.seek(std::io::SeekFrom::Start(0)).is_err() {
            return Err(DiskImageError::FormatParseError);
        }
    
        let rwr = match load::<RecoveryWhiskersRaw>(&mut image, RWR_VER) {
            Ok(data) => data,
            Err(_) => return Err(DiskImageError::FormatParseError),
        };

        let mut disk_data_rate = None;

        // group flux streams by physical positions
        // NOTE: original parsed location info is ignored
        // NOTE: only full(index to index) revolutions are kept
        let mut by_pos: BTreeMap<(u8, u16), Vec<&FluxPart>> = BTreeMap::new();
        for flux in rwr.fluxes.iter() {
            let pos = flux.position;
            by_pos
                .entry(pos)
                .or_default()
                .extend(flux.parts.iter().filter(|p| p.rotation == Rotation::Full));
        }

        let max_tracks = by_pos.keys().map(|(_, t)| *t).max().unwrap_or_default() + 1;
        let side_count = by_pos.keys().map(|(s, _)| *s).max().unwrap_or_default() + 1;
        log::info!("RecoveryWhiskersRaw: Total physical positions count: {}", by_pos.len());
        log::info!("RecoveryWhiskersRaw: Full revolutions count: {}", by_pos.values().map(|v|v.len()).sum::<usize>());
        log::info!("RecoveryWhiskersRaw: Max revolutions for a single track: {}", by_pos.values().map(|v|v.len()).max().unwrap_or_default());

        // RWR stores original recorded time duration expressed in Greaseweazle sample ticks in FluxPart.data
        for ((side, track), flux_parts) in by_pos {
            // small terminology differences :/.
            let ch = DiskCh::new(track, side);
            let mut flux_track = FluxStreamTrack::new();

            log::trace!("RecoveryWhiskersRaw: Processing side {} track {}", side, track);

            for rev in flux_parts.iter() {
                let sample_freq = rev.freq;
                let mut fluxes: Vec<f64> = Vec::with_capacity(rev.data.len());
                let mut accumulator: u64 = 0;
                let mut big_data_position: usize = 0;

                // handle large gaps and calculate RPM
                // convert on the fly
                for (i, &interval) in rev.data.iter().enumerate() {
                    let duation: u64 = if interval == u16::MAX
                        && rev.big_data.len() > big_data_position
                        && rev.big_data[big_data_position].0 == (i as u64 + 1)
                    {
                        // maybe a big gap recorded as u32
                        // big_data is ordered
                        let interval = rev.big_data[big_data_position].1 as u64;
                        big_data_position += 1;
                        interval
                        
                    } else {
                        interval as u64
                    };

                    accumulator += duation;

                    // convert flux data to f64 seconds
                    fluxes.push(interval as f64 / sample_freq);
                }

                // wait, where's the index time? rwr throws it away :(
                // recovery whiskers doesn't rely on RPM to recover data
                // hmm it seems not important
                let rev_seconds = accumulator as f64 / sample_freq;
                flux_track.add_revolution(ch, &fluxes, rev_seconds);
            }

            let params = FluxStreamTrackParams {
                ch,
                schema: None,
                encoding: None,
                clock: None,
                rpm: None,
            };

            let new_track = disk_image.add_track_fluxstream(flux_track, &params)?;

            let ti = new_track.info();

            if disk_data_rate.is_none() {
                log::trace!("Setting disk data rate to {}", ti.data_rate);
                disk_data_rate = Some(ti.data_rate);
            }
        }

        if disk_data_rate.is_none() {
            log::error!("Unable to determine data rate from any track.");
            return Err(DiskImageError::IncompatibleImage(
                "Unable to determine data rate from any track.".to_string(),
            ));
        }

        disk_image.descriptor = DiskDescriptor {
            // RWR doesn't specify platform at all. Figure it out after import.
            platforms: None,
            // required to visualize properly
            // but I don't think it's the job for raw flux formats...
            geometry: DiskCh::from((max_tracks, side_count)),
            data_rate: disk_data_rate.unwrap(),
            density: TrackDensity::from(disk_data_rate.unwrap()),
            data_encoding: TrackDataEncoding::Mfm,
            rpm: None,
            write_protect: Some(true),
        };

        log::info!("RecoveryWhiskersRaw: done loading");

        Ok(())
    }

    pub fn save_image<RWS: ReadWriteSeek>(
        _image: &DiskImage,
        _opts: &ParserWriteOptions,
        _output: &mut RWS,
    ) -> Result<(), DiskImageError> {
        Err(DiskImageError::UnsupportedFormat)
    }
}
