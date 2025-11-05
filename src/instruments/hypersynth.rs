use super::common::SynthParams;
use super::common::TranspEq;
use super::common::COMMON_FILTER_TYPES;
use super::dests;
use super::CommandPack;
use super::Version;
use crate::reader::*;
use crate::writer::Writer;
use crate::FIRMWARE_6_0_SONG_VERSION;
use crate::FIRMWARE_6_2_SONG_VERSION;
use crate::SEND_COMMAND_NAMES;
use crate::SEND_COMMAND_NAMES_6_2;

use arr_macro::arr;
use array_concat::concat_arrays;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Chord {
    pub mask: u8,
    pub offsets : [u8; 6]
}

impl Chord {
    pub fn read(reader: &mut Reader) -> Self{
        let mask = reader.read();

        Self {
            mask,
            offsets: arr![reader.read(); 6]
        }
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.mask);
        for k in &self.offsets {
            w.write(*k);
        }
    }

    pub fn is_osc_on(&self, osc: usize) -> bool{
        (self.mask & (1 << osc)) != 0
    }

    pub fn offset_str(&self, osc: usize) -> String {
        if self.is_osc_on(osc) {
            format!("{:02X}", self.offsets[osc])
        } else {
            String::from("--")
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct HyperSynth {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub scale: u8,
    pub default_chord: [u8; 7],
    pub shift: u8,
    pub swarm: u8,
    pub width: u8,
    pub subosc: u8,

    pub chords: [Chord; 0x10]
}

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const HYPERSYNTH_COMMAND_NAMES_BASE : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT - 3] = [
    "VOL",
    "PIT",
    "FIN",
    "CRD",
    "SHF",
    "SWM",
    "WID",
    "SUB",
    "FLT",
    "CUT",
    "RES",
    "AMP",
    "LIM",
    "PAN",
    "DRY"
];
    

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const HYPERSYNTH_COMMAND_NAMES : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    concat_arrays!(HYPERSYNTH_COMMAND_NAMES_BASE, SEND_COMMAND_NAMES, ["CVO", "SNC"]);

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const HYPERSYNTH_COMMAND_NAMES_BASE_6 : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT - 3] = [
    "VOL",
    "PIT",
    "FIN",
    "CRD",
    "CVO",
    "SWM",
    "WID",
    "SUB",
    "FLT",
    "CUT",
    "RES",
    "AMP",
    "LIM",
    "PAN",
    "DRY"
];
    

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const HYPERSYNTH_COMMAND_NAMES_6 : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    concat_arrays!(HYPERSYNTH_COMMAND_NAMES_BASE_6, SEND_COMMAND_NAMES, ["SNC", "ERR"]);

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const HYPERSYNTH_COMMAND_NAMES_6_2 : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    concat_arrays!(HYPERSYNTH_COMMAND_NAMES_BASE_6, SEND_COMMAND_NAMES_6_2, ["SNC", "ERR"]);

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const DESTINATIONS : [&'static str; 15] = [
    dests::OFF,
    dests::VOLUME,
    dests::PITCH,

    "SHIFT",
    "SWARM",
    "WIDTH",
    "SUBOSC",
    dests::CUTOFF,
    dests::RES,
    dests::AMP,
    dests::PAN,
    dests::MOD_AMT,
    dests::MOD_RATE,
    dests::MOD_BOTH,
    dests::MOD_BINV,
];

impl HyperSynth {
    const MOD_OFFSET: usize = 23;

    pub fn command_name(&self, ver: Version) -> &'static [&'static str] {
        if ver.after(&FIRMWARE_6_2_SONG_VERSION) {
            &HYPERSYNTH_COMMAND_NAMES_6_2
        } else if ver.after(&FIRMWARE_6_0_SONG_VERSION) {
            &HYPERSYNTH_COMMAND_NAMES_6
        } else {
            &HYPERSYNTH_COMMAND_NAMES
        }
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    /// List of all the applyable filter types for the instrument
    pub fn filter_types(&self, _ver: Version) -> &'static [&'static str] {
        &super::common::COMMON_FILTER_TYPES
    }

    pub fn human_readable_filter(&self) -> &'static str {
        COMMON_FILTER_TYPES[self.synth_params.filter_type as usize]
    }

    pub fn write(&self, ver: Version, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(TranspEq::from(ver, self.transpose, self.synth_params.associated_eq).into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        for c in self.default_chord {
            w.write(c);
        }

        w.write(self.scale);
        w.write(self.shift);
        w.write(self.swarm);
        w.write(self.width);
        w.write(self.subosc);

        self.synth_params.write(ver, w, HyperSynth::MOD_OFFSET);

        for chd in &self.chords {
            chd.write(w);
        }
    }

    pub fn from_reader(ver: Version, reader: &mut Reader, number: u8) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = TranspEq::from_version(ver, reader.read());
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let default_chord = arr![reader.read(); 7];
        let scale = reader.read();
        let shift = reader.read();
        let swarm = reader.read();
        let width = reader.read();
        let subosc = reader.read();
        let synth_params = SynthParams::from_reader3(
            ver,
            reader,
            volume,
            pitch,
            fine_tune,
            transp_eq.eq,
            HyperSynth::MOD_OFFSET,
        )?;

        let chords = arr![Chord::read(reader); 0x10];

        Ok(HyperSynth {
            number,
            name,
            transpose: transp_eq.transpose,
            table_tick,
            synth_params,

            scale,
            default_chord,
            shift,
            swarm,
            width,
            subosc,
            chords
        })
    }
}
