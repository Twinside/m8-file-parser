use crate::*;

/// Interface to gather and display parameters in a semi
/// automated manner
pub trait ParameterGatherer {
    /// Display a hex value for the instrument
    fn hex(self, name: &str, val: u8) -> Self;

    /// Display a boolean value for the described element
    fn bool(self, name: &str, val: bool) -> Self;

    /// Display a floating point value.
    fn float(self, name: &str, val: f64) -> Self;

    /// Display a string
    fn str(self, name: &str, val: &str) -> Self;

    /// Write an enumeration, with an hex code and a string representation
    /// alongside it.
    fn enumeration(self, name: &str, hex: u8, val: &str) -> Self;

    /// Enter a sub scope, the callback should use the nested gatherer
    /// to write the arguments.
    fn nest_f<F>(self, name: &str, f: F) -> Self
        where F : FnOnce (Self) -> Self, Self : Sized;
}

/// Interface implementing generic description of M8 structures
/// for human consumption.
pub trait Describable {
    /// Method called to describte the content of the structure in any gatherer.
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG;
}

/// Some pretty printing require a dictionary found elsewhere. Encode that
pub trait DescribableWithDictionary {
    fn describe_with_dic< PG : ParameterGatherer>(&self, pg: PG, dic: &[&'static str], ver: Version) -> PG;
}

impl Describable for EqBand {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, _ver: Version) -> PG {
        return pg.float("GAIN", self.gain())
          .float("FREQ", self.frequency() as f64)
          .hex("Q", self.q)
          .enumeration("TYPE", self.mode.eq_type_hex(), self.mode.type_str())
          .enumeration("MODE", self.mode.eq_mode_hex(), self.mode.mode_str())
    }
}

impl Describable for Equ {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        return pg
            .nest_f("LOW", |ii| self.low.describe(ii, ver))
            .nest_f("MID", |ii| self.mid.describe(ii, ver))
            .nest_f("HIGH", |ii| self.high.describe(ii, ver));
    }
}

impl Describable for Operator {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, _ver: Version) -> PG{
        return pg
          .str("SHAPE", &format!("{:?}", self.shape))
          .float("RATIO", (self.ratio as f64) + (self.ratio_fine as f64) / 100.0)
          .hex("LEVEL", self.level)
          .hex("FBK", self.feedback)
          .hex("MOD_A", self.mod_a)
          .hex("MOD_B", self.mod_b);
    }
}

impl Describable for FMSynth {
   fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let pg = pg
            .str(params::NAME, &self.name)
            .bool(params::TRANSPOSE, self.transpose)
            .hex(params::EQ, self.synth_params.associated_eq)
            .hex(params::TBLTIC, self.table_tick)
            .enumeration("ALG", self.algo.0, self.algo.str())
            .nest_f("A", |ipg| self.operators[0].describe(ipg, ver))
            .nest_f("B", |ipg| self.operators[1].describe(ipg, ver))
            .nest_f("C", |ipg| self.operators[2].describe(ipg, ver))
            .nest_f("D", |ipg| self.operators[3].describe(ipg,ver));

        let pg = self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        return describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
   }
}

impl Describable for Sampler {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let pg = pg
            .str(params::NAME, &self.name)
            .bool(params::TRANSPOSE, self.transpose)
            .hex(params::TBLTIC, self.table_tick)
            .hex(params::EQ, self.synth_params.associated_eq)
            .str("SAMPLE", &self.sample_path)
            .enumeration("PLAY", self.play_mode as u8, &format!("{:?}", self.play_mode))
            .hex("SLICE", self.slice);

        let pg = match self.play_mode {
            SamplePlayMode::FWD |
            SamplePlayMode::REV |
            SamplePlayMode::FWDLOOP |
            SamplePlayMode::REVLOOP |
            SamplePlayMode::FWD_PP |
            SamplePlayMode::REV_PP |
            SamplePlayMode::OSC |
            SamplePlayMode::OSC_REV |
            SamplePlayMode::OSC_PP => {
                pg.hex("START", self.start)
                  .hex("LOOP ST", self.loop_start)
                  .hex("LENGTH", self.length)
                  .hex("DETUNE", self.synth_params.pitch)
            },

            SamplePlayMode::REPITCH |
            SamplePlayMode::REP_REV |
            SamplePlayMode::REP_PP => {
                pg.hex("STEPS", self.synth_params.pitch)
                  .hex("START", self.start)
                  .hex("LOOP ST", self.loop_start)
                  .hex("LENGTH", self.length)
            },

            SamplePlayMode::REP_BPM |
            SamplePlayMode::BPM_REV |
            SamplePlayMode::BPM_PP => {
                pg.hex("BPM", self.synth_params.pitch)
                  .hex("START", self.start)
                  .hex("LOOP ST", self.loop_start)
                  .hex("LENGTH", self.length)
            }
        };

        let pg =
            pg.hex("DEGRADE", self.degrade);

        let pg = self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver)
    }
}

impl Describable for WavSynth {
   fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let pg = pg
          .str(params::NAME, &self.name)
          .bool(params::TRANSPOSE, self.transpose)
          .hex(params::TBLTIC, self.table_tick)
          .hex(params::EQ, self.synth_params.associated_eq)
          .enumeration("SHAPE", self.shape as u8, &format!("{:?}", self.shape))
          .hex("SIZE", self.size)
          .hex("MULT", self.mult)
          .hex("WARP", self.warp)
          .hex("SCAN", self.scan);

        let pg =
            self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);

        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver)
   }
}

impl Describable for Instrument {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        match self {
            Instrument::WavSynth(ws)     => 
                pg.nest_f("WAVSYNTH", |ipg| ws.describe(ipg, ver)),
            Instrument::MacroSynth(ms) =>
                pg.nest_f("MACROSYN", |ipg| ms.describe(ipg, ver)),
            Instrument::Sampler(s)        =>
                pg.nest_f("SAMPLE", |ipg| s.describe(ipg, ver)),
            Instrument::MIDIOut(mo)       =>
                pg.nest_f("MIDIOUT", |ipg| mo.describe(ipg, ver)),
            Instrument::FMSynth(fs)       =>
                pg.nest_f("FMSYNTH", |ipg| fs.describe(ipg, ver)),
            Instrument::HyperSynth(hs) =>
                pg.nest_f("HYPERSYNTH", |ipg| hs.describe(ipg, ver)),
            Instrument::External(ex) =>
                pg.nest_f("EXTERNALINST", |ipg| ex.describe(ipg, ver)),
            Instrument::None => pg
        }
    }
}

pub fn describe_succint<PG : ParameterGatherer>(instr: &Instrument, pg: PG, ver: Version) -> PG {
    let (k, common) =
        match instr {
            Instrument::WavSynth(ws)     => ("WAVSYNTH", Some(&ws.synth_params)),
            Instrument::MacroSynth(ms) => ("MACROSYN", Some(&ms.synth_params)),
            Instrument::Sampler(s)        => ("SAMPLE", Some(&s.synth_params)),
            Instrument::MIDIOut(_mo)       => ("MIDIOUT", None),
            Instrument::FMSynth(fs)       => ("FMSYNTH", Some(&fs.synth_params)),
            Instrument::HyperSynth(hs) => ("HYPERSYNTH", Some(&hs.synth_params)),
            Instrument::External(ex) => ("EXTERNALINST", Some(&ex.synth_params)),
            Instrument::None => ("NONE", None)
        };

    let pg = pg.str("KIND", k);
    match common {
        None => pg,
        Some(c) => describe_succint_params(&c, pg, ver)
    }
}

impl Describable for ExternalInst {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let port_str = self.human_readable_port();
        let pg = pg
          .str(params::NAME, &self.name)
          .bool(params::TRANSPOSE, self.transpose)
          .hex(params::EQ, self.synth_params.associated_eq)
          .hex(params::TBLTIC, self.table_tick)

          .enumeration("PORT", self.port, port_str)
          .hex("CHANNEL", self.channel)
          .hex("BANK", self.bank)
          .hex("PROGRAM", self.program)
          .nest_f(params::CCA, |ipg| self.cca.describe(ipg, ver))
          .nest_f(params::CCB, |ipg| self.ccb.describe(ipg, ver))
          .nest_f(params::CCC, |ipg| self.ccc.describe(ipg, ver))
          .nest_f(params::CCD, |ipg| self.ccd.describe(ipg, ver));

        let pg =
            self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);

        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver)
    }
}

impl Describable for HyperSynth {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let dc = &self.default_chord;

        let pg = pg
          .str(params::NAME, &self.name)
          .bool(params::TRANSPOSE, self.transpose)
          .hex(params::EQ, self.synth_params.associated_eq)
          .hex(params::SCALE, self.scale)
          .str("CHORD", &format!("{:02X} | {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", dc[0], dc[1], dc[2], dc[3], dc[4], dc[5], dc[6]))
          .hex(params::TBLTIC, self.table_tick)
          .hex("SHIFT", self.shift)
          .hex("SWARM", self.swarm)
          .hex("WIDTH", self.width)
          .hex("SUBOSC", self.subosc);

        let pg =
            self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver)
    }
}

impl Describable for MacroSynth {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let pg = pg
          .str(params::NAME, &self.name)
          .bool(params::TRANSPOSE, self.transpose)
          .hex(params::EQ, self.synth_params.associated_eq)
          .hex(params::TBLTIC, self.table_tick)
          .enumeration("SHAPE", self.shape as u8, &format!("{:?}", self.shape))
          .hex("TIMBRE", self.timbre)
          .hex("COLOR", self.color)
          .hex("DEGRADE", self.degrade)
          .hex("REDUX", self.redux);

        let pg =
            self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);

        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver)
    }
}

impl Describable for ControlChange {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, _ver: Version) -> PG {
        return pg
          .hex("CC", self.number)
          .hex("VAL", self.value);
    }
}

impl Describable for MIDIOut {
    fn describe<PG : ParameterGatherer>(&self, pg: PG, ver: Version) -> PG {
        let port_str = self.human_readable_port();
        let pg = pg
          .str(params::NAME, &self.name)
          .bool(params::TRANSPOSE, self.transpose)
          .hex(params::TBLTIC, self.table_tick)

          .enumeration("PORT", self.port, port_str)
          .hex("CHANNEL", self.channel)
          .hex("BANK", self.bank_select)
          .hex("PROGRAM", self.program_change)
          .nest_f("CCA", |ipg| self.custom_cc[0].describe(ipg, ver))
          .nest_f("CCB", |ipg| self.custom_cc[1].describe(ipg, ver))
          .nest_f("CCC", |ipg| self.custom_cc[2].describe(ipg, ver))
          .nest_f("CCD", |ipg| self.custom_cc[3].describe(ipg, ver))
          .nest_f("CCE", |ipg| self.custom_cc[4].describe(ipg, ver))
          .nest_f("CCF", |ipg| self.custom_cc[5].describe(ipg, ver))
          .nest_f("CCG", |ipg| self.custom_cc[6].describe(ipg, ver))
          .nest_f("CCH", |ipg| self.custom_cc[7].describe(ipg, ver))
          .nest_f("CCI", |ipg| self.custom_cc[8].describe(ipg, ver))
          .nest_f("CCJ", |ipg| self.custom_cc[9].describe(ipg, ver));

        describe_modulators(&self.mods, pg, self.destination_names(ver), ver)
    }
}

impl DescribableWithDictionary for ADSREnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .hex(params::AMOUNT, self.amount)
          .hex(params::ATTACK, self.attack)
          .hex(params::DECAY, self.decay)
          .hex(params::SUSTAIN, self.sustain)
          .hex(params::RELEASE, self.release);
    }
}

impl DescribableWithDictionary for AHDEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG{
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .hex(params::AMOUNT, self.amount)
          .hex(params::ATTACK, self.attack)
          .hex(params::HOLD, self.hold)
          .hex(params::DECAY, self.decay);
    }
}

impl DescribableWithDictionary for DrumEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .hex(params::AMOUNT, self.amount)
          .hex(params::PEAK, self.peak)
          .hex(params::BODY, self.body)
          .hex(params::DECAY, self.decay);
    }
}

impl DescribableWithDictionary for LFO {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .enumeration(params::LFOSHAPE, self.shape as u8, &format!("{:?}", self.shape))
          .hex(params::AMOUNT, self.amount)
          .hex(params::FREQ, self.freq)
          .enumeration(params::TRIGGER, self.shape as u8, &format!("{:?}", self.trigger_mode));
    }
}

impl DescribableWithDictionary for TrackingEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .hex(params::AMOUNT, self.amount)
          .hex(params::SOURCE, self.src)
          .hex("LVAL", self.lval)
          .hex("HVAL", self.hval);
    }
}

impl DescribableWithDictionary for TrigEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, dests: &[&'static str], _ver: Version) -> PG {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        return pg
          .enumeration(params::DEST, self.dest, dest_str)
          .hex(params::AMOUNT, self.amount)
          .hex(params::ATTACK, self.attack)
          .hex(params::HOLD, self.hold)
          .str(params::SOURCE, &format!("{:?}", self.src));
    }
}

fn describe_mod<PG : ParameterGatherer>(modulator: &Mod, pg: PG, ix: usize, dests:&[&'static str], ver: Version) -> PG {
    let ix = ix + 1;
    match modulator {
        Mod::AHDEnv(ahd)  => {
            let pg = pg.enumeration(&format!("MOD{ix}"), 0, "AHD ENV");
            ahd.describe_with_dic(pg, dests, ver)
        },
        Mod::ADSREnv(adsr) => {
            let pg = pg.enumeration(&format!("MOD{ix}"), 1, "ADSR ENV");
            adsr.describe_with_dic(pg, dests, ver)
        },
        Mod::DrumEnv(drum_env) =>{
            let pg = pg.enumeration(&format!("MOD{ix}"), 1, "DRUM ENV");
            drum_env.describe_with_dic(pg, dests, ver)
        }
        Mod::LFO(lfo) => {
            let pg = pg.enumeration(&format!("MOD{ix}"), 1, "LFO");
            lfo.describe_with_dic(pg, dests, ver)
        }
        Mod::TrigEnv(tenv) => {
            let pg = pg.enumeration(&format!("MOD{ix}"), 1, "TRIGENV");
            tenv.describe_with_dic(pg, dests, ver)
        }
        Mod::TrackingEnv(tenv) => {
            let pg = pg.enumeration(&format!("MOD{ix}"), 1, "TRACKENV");
            tenv.describe_with_dic(pg, dests, ver)
        },
    }
}

pub fn describe_modulators<PG : ParameterGatherer>(sp: &SynthParams, pg: PG, dests: &[&'static str], ver: Version) -> PG {
    return pg
        .nest_f("MOD1", |ipg| describe_mod(&sp.mods[0], ipg, 0, dests, ver))
        .nest_f("MOD2", |ipg| describe_mod(&sp.mods[1], ipg, 1, dests, ver))
        .nest_f("MOD3", |ipg| describe_mod(&sp.mods[2], ipg, 2, dests, ver))
        .nest_f("MOD4", |ipg| describe_mod(&sp.mods[3], ipg, 3, dests, ver));
}

pub fn describe_succint_params<PG : ParameterGatherer>(sp: &SynthParams, pg: PG, _ver: Version) -> PG{
    return pg
      .hex(params::EQ, sp.associated_eq)
      .hex(dests::AMP, sp.amp)
      .enumeration("LIM", sp.limit.0, sp.limit.str())
      .hex(dests::PAN, sp.mixer_pan)
      .hex("DRY", sp.mixer_dry)
      .hex("CHORUS", sp.mixer_chorus)
      .hex("DELAY", sp.mixer_delay)
      .hex("REVERB", sp.mixer_reverb);
}

impl DescribableWithDictionary for SynthParams {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: PG, filters: &[&str], ver: Version) -> PG {
        let pg = pg.hex("FINE", self.fine_tune);

        let pg =
            match filters.get(self.filter_type as usize) {
                None =>
                    pg.enumeration("FILTER", self.filter_type, &format!("{:02X}", self.filter_type)),
                Some(str) => 
                    pg.enumeration("FILTER", self.filter_type, str)
            };

        let pg = pg
            .hex("CUT", self.filter_cutoff)
            .hex("RES", self.filter_res);

        describe_succint_params(self, pg, ver)
    }
}
