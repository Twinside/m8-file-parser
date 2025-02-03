use crate::reader::*;
use super::common::SynthParams;
use super::common::TranspEq;
use super::common::COMMON_FILTER_TYPES;
use super::dests;
use super::params;
use super::CommandPack;
use super::ParameterGatherer;
use super::Version;

use arr_macro::arr;

#[derive(PartialEq, Debug, Clone)]
pub struct HyperSynth {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub scale: u8,
    pub default_chord: [u8; 7],
    pub shift: u8,
    pub swarm: u8,
    pub width: u8,
    pub subosc: u8,

    pub chords: [[u8; 6]; 0x10]
}

const HYPERSYNTH_COMMAND_NAMES : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    [
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
      "DRY",
      
      "SCH",
      "SDL",
      "SRV",

      // EXTRA
      "CVO",
      "SNC"
    ];

const DESTINATIONS : [&'static str; 15] =
    [
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
    const MOD_OFFSET : usize = 23;

    pub fn command_name(&self, _ver : Version) -> &'static[&'static str] {
        &HYPERSYNTH_COMMAND_NAMES 
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transp_eq.transpose);
        pg.hex(params::EQ, self.transp_eq.eq);
        pg.hex(params::SCALE, self.scale);
        let dc = &self.default_chord;
        pg.str("CHORD", &format!("{:02X} | {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", dc[0], dc[1], dc[2], dc[3], dc[4], dc[5], dc[6]));
        pg.hex(params::TBLTIC, self.table_tick);
        pg.hex("SHIFT", self.shift);
        pg.hex("SWARM", self.swarm);
        pg.hex("WIDTH", self.width);
        pg.hex("SUBOSC", self.subosc);

        self.synth_params.describe(pg, &COMMON_FILTER_TYPES);
        self.synth_params.describe_modulators(pg, self.destination_names(ver));

        // TODO: other chords
    }

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
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

        self.synth_params.write(w, HyperSynth::MOD_OFFSET);

        for chd in self.chords {
            w.write(0xFF);
            for k in chd { w.write(k); }
        }
    }

    fn load_chord(reader: &mut Reader) -> [u8; 6] {
        // padding
        let _ = reader.read();
        arr![reader.read(); 6]
    }

    pub fn from_reader(reader: &mut Reader, number: u8) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
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
        let synth_params =
            SynthParams::from_reader3(reader, volume, pitch, fine_tune, HyperSynth::MOD_OFFSET)?;

        let chords =
            arr![HyperSynth::load_chord(reader); 0x10];

        Ok(HyperSynth {
            number,
            name,
            transp_eq,
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
