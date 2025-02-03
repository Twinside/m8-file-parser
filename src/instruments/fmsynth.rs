use crate::reader::*;
use crate::version::*;
use crate::instruments::common::*;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use arr_macro::arr;

use super::dests;
use super::params;
use super::CommandPack;
use super::ParameterGatherer;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FmAlgo(u8);

const FM_ALGO_STRINGS : [&str; 0x0C] =
    [
        "A>B>C>D",
        "[A+B]>C>D",
        "[A>B+C]>D",
        "[A>B+A>C]>D",
        "[A+B+C]>D",
        "[A>B>C]+D",
        "[A>B>C]+[A>B>D]",
        "[A>B]+[C>D]",
        "[A>B]+[A>C]+[A>D]",
        "[A>B]+[A>C]+D",
        "[A>B]+C+D",
        "A+B+C+D"
    ];

impl TryFrom<u8> for FmAlgo {
    type Error = ParseError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        if (value as usize) < FM_ALGO_STRINGS.len() {
            Ok(FmAlgo(value))
        } else {
            Err(ParseError(format!("Invalid fm algo {}", value)))
        }
    }
}

impl FmAlgo {
    pub fn id(self) -> u8 {
        let FmAlgo(v) = self;
        v
    }

    pub fn str(self) -> &'static str {
        FM_ALGO_STRINGS[self.id() as usize]
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum FMWave {
    #[default]
    SIN,
    SW2,
    SW3,
    SW4,
    SW5,
    SW6,
    TRI,
    SAW,
    SQR,
    PUL,
    IMP,
    NOI,
    NLP,
    NHP,
    NBP,
    CLK
}

const FM_FX_COMMANDS : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 1] =
  [
    "VOL",
    "PIT",
    "FIN",
    "ALG",
    "FM1",
    "FM2",
    "FM3",
    "FM4",
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
    
    "FMP",
  ];

const DESTINATIONS : [&'static str; 15] =
    [
        dests::OFF,
        dests::VOLUME,
        dests::PITCH,

        "MOD1",
        "MOD2",
        "MOD3",
        "MOD4",
        dests::CUTOFF,
        dests::RES,
        dests::AMP,
        dests::PAN,
        dests::MOD_AMT,
        dests::MOD_RATE,
        dests::MOD_BOTH,
        dests::MOD_BINV,
    ];

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Operator {
    pub shape: FMWave,
    pub ratio: u8,
    pub ratio_fine: u8,
    pub level: u8,
    pub feedback: u8,
    pub retrigger: u8,
    pub mod_a: u8,
    pub mod_b: u8,
}

impl Operator {
    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG) {
        pg.str("SHAPE", &format!("{:?}", self.shape));
        pg.float("RATIO", (self.ratio as f64) + (self.ratio_fine as f64) / 100.0);
        pg.hex("LEVEL", self.level);
        pg.hex("FBK", self.feedback);
        pg.hex("MOD_A", self.mod_a);
        pg.hex("MOD_B", self.mod_b);
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct FMSynth {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub algo: FmAlgo,
    pub operators: [Operator; 4],
    pub mod1: u8,
    pub mod2: u8,
    pub mod3: u8,
    pub mod4: u8,
}

impl FMSynth {
    const MOD_OFFSET : usize = 2;

    pub fn command_name(&self, _ver: Version) -> &'static [&'static str] {
        &FM_FX_COMMANDS
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transp_eq.transpose);
        pg.hex(params::EQ, self.transp_eq.eq);
        pg.hex(params::TBLTIC, self.table_tick);
        pg.enumeration("ALG", self.algo.0, self.algo.str());

        self.operators[0].describe(&mut pg.nest("A"));
        self.operators[1].describe(&mut pg.nest("B"));
        self.operators[2].describe(&mut pg.nest("C"));
        self.operators[3].describe(&mut pg.nest("D"));

        self.synth_params.describe(pg, &COMMON_FILTER_TYPES);
        self.synth_params.describe_modulators(pg, self.destination_names(ver));
    }

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.algo.0);

        for op in &self.operators {
            w.write(op.shape.into());
        }

        for op in &self.operators {
            w.write(op.ratio);
            w.write(op.ratio_fine);
        }

        for op in &self.operators {
            w.write(op.level);
            w.write(op.feedback);
        }

        for op in &self.operators {
            w.write(op.mod_a);
        }

        for op in &self.operators {
            w.write(op.mod_b);
        }

        w.write(self.mod1);
        w.write(self.mod2);
        w.write(self.mod3);
        w.write(self.mod4);

        self.synth_params.write(w, FMSynth::MOD_OFFSET);
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let algo = reader.read();
        let mut operators: [Operator; 4] = arr![Operator::default(); 4];
        if version.at_least(1, 4) {
            for i in 0..4 {
                let wav_code = reader.read();
                operators[i].shape = FMWave::try_from(wav_code)
                    .map_err(|_| ParseError(format!("Invalid fm wave {}", wav_code)))?;
            }
        }
        for i in 0..4 {
            operators[i].ratio = reader.read();
            operators[i].ratio_fine = reader.read();
        }
        for i in 0..4 {
            operators[i].level = reader.read();
            operators[i].feedback = reader.read();
        }
        for i in 0..4 {
            operators[i].mod_a = reader.read();
        }
        for i in 0..4 {
            operators[i].mod_b = reader.read();
        }
        let mod1 = reader.read();
        let mod2 = reader.read();
        let mod3 = reader.read();
        let mod4 = reader.read();

        let synth_params =
            if version.at_least(3, 0) {
                SynthParams::from_reader3(reader, volume, pitch, fine_tune, FMSynth::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        Ok(FMSynth {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            algo: FmAlgo(algo),
            operators,
            mod1,
            mod2,
            mod3,
            mod4,
        })
    }
}
