use crate::*;

/// Interface to gather and display parameters in a semi
/// automated manner
pub trait ParameterGatherer {
    /// Display a hex value for the instrument
    fn hex(&mut self, name: &str, val: u8);

    /// Display a boolean value for the described element
    fn bool(&mut self, name: &str, val: bool);

    /// Display a floating point value.
    fn float(&mut self, name: &str, val: f64);

    /// Display a string
    fn str(&mut self, name: &str, val: &str);

    /// Write an enumeration, with an hex code and a string representation
    /// alongside it.
    fn enumeration(&mut self, name: &str, hex: u8, val: &str);

    /// Enter a sub scope, the returned object represent the subscope
    /// where you can write new parameters.
    fn nest(&mut self, name: &str) -> Self;
}

/// Interface implementing generic description of M8 structures
/// for human consumption.
pub trait Describable {
    /// Method called to describte the content of the structure in any gatherer.
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version);
}

/// Some pretty printing require a dictionary found elsewhere. Encode that
pub trait DescribableWithDictionary {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dic: &[&'static str], ver: Version);
}

impl Describable for EqBand {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, _ver: Version) {
        pg.float("GAIN", self.gain());
        pg.float("FREQ", self.frequency() as f64);
        pg.hex("Q", self.q);
        pg.enumeration("TYPE", self.mode.eq_type_hex(), self.mode.type_str());
        pg.enumeration("MODE", self.mode.eq_mode_hex(), self.mode.mode_str());
    }
}

impl Describable for Equ {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        self.low.describe(&mut pg.nest("LOW"), ver);
        self.mid.describe(&mut pg.nest("MID"), ver);
        self.high.describe(&mut pg.nest("HIGH"), ver);
    }
}

impl Describable for Operator {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, _ver: Version) {
        pg.str("SHAPE", &format!("{:?}", self.shape));
        pg.float("RATIO", (self.ratio as f64) + (self.ratio_fine as f64) / 100.0);
        pg.hex("LEVEL", self.level);
        pg.hex("FBK", self.feedback);
        pg.hex("MOD_A", self.mod_a);
        pg.hex("MOD_B", self.mod_b);
    }
}

impl Describable for FMSynth {
   fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.hex(params::TBLTIC, self.table_tick);
        pg.enumeration("ALG", self.algo.0, self.algo.str());

        self.operators[0].describe(&mut pg.nest("A"), ver);
        self.operators[1].describe(&mut pg.nest("B"), ver);
        self.operators[2].describe(&mut pg.nest("C"), ver);
        self.operators[3].describe(&mut pg.nest("D"), ver);

        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
   }
}

impl Describable for Sampler {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::TBLTIC, self.table_tick);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.str("SAMPLE", &self.sample_path);
        pg.enumeration("PLAY", self.play_mode as u8, &format!("{:?}", self.play_mode));

        pg.hex("SLICE", self.slice);
        match self.play_mode {
            SamplePlayMode::FWD |
            SamplePlayMode::REV |
            SamplePlayMode::FWDLOOP |
            SamplePlayMode::REVLOOP |
            SamplePlayMode::FWD_PP |
            SamplePlayMode::REV_PP |
            SamplePlayMode::OSC |
            SamplePlayMode::OSC_REV |
            SamplePlayMode::OSC_PP => {
                pg.hex("START", self.start);
                pg.hex("LOOP ST", self.loop_start);
                pg.hex("LENGTH", self.length);
                pg.hex("DETUNE", self.synth_params.pitch);
            },

            SamplePlayMode::REPITCH |
            SamplePlayMode::REP_REV |
            SamplePlayMode::REP_PP => {
                pg.hex("STEPS", self.synth_params.pitch);
                pg.hex("START", self.start);
                pg.hex("LOOP ST", self.loop_start);
                pg.hex("LENGTH", self.length);
            },

            SamplePlayMode::REP_BPM |
            SamplePlayMode::BPM_REV |
            SamplePlayMode::BPM_PP => {
                pg.hex("BPM", self.synth_params.pitch);
                pg.hex("START", self.start);
                pg.hex("LOOP ST", self.loop_start);
                pg.hex("LENGTH", self.length);
            }
        }

        pg.hex("DEGRADE", self.degrade);

        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
    }
}

impl Describable for WavSynth {
   fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::TBLTIC, self.table_tick);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.enumeration("SHAPE", self.shape as u8, &format!("{:?}", self.shape));
        pg.hex("SIZE", self.size);
        pg.hex("MULT", self.mult);
        pg.hex("WARP", self.warp);
        pg.hex("SCAN", self.scan);

        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
   }
}

impl Describable for Instrument {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        match self {
            Instrument::WavSynth(ws)     => ws.describe(&mut pg.nest("WAVSYNTH"), ver),
            Instrument::MacroSynth(ms) => ms.describe(&mut pg.nest("MACROSYN"), ver),
            Instrument::Sampler(s)        => s.describe(&mut pg.nest("SAMPLE"), ver),
            Instrument::MIDIOut(mo)       => mo.describe(&mut pg.nest("MIDIOUT"), ver),
            Instrument::FMSynth(fs)       => fs.describe(&mut pg.nest("FMSYNTH"), ver),
            Instrument::HyperSynth(hs) => hs.describe(&mut pg.nest("HYPERSYNTH"), ver),
            Instrument::External(ex) => ex.describe(&mut pg.nest("EXTERNALINST"), ver),
            Instrument::None => {}
        };
    }
}

pub fn describe_succint<PG : ParameterGatherer>(instr: &Instrument, pg: &mut PG, ver: Version) {
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

    pg.str("KIND", k);
    match common {
        None => {}
        Some(c) => describe_succint_params(&c, pg, ver)
    }
}

impl Describable for ExternalInst {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.hex(params::TBLTIC, self.table_tick);

        let port_str = self.human_readable_port();
        pg.enumeration("PORT", self.port, port_str);
        pg.hex("CHANNEL", self.channel);
        pg.hex("BANK", self.bank);
        pg.hex("PROGRAM", self.program);
        self.cca.describe(&mut pg.nest(params::CCA), ver);
        self.ccb.describe(&mut pg.nest(params::CCB), ver);
        self.ccc.describe(&mut pg.nest(params::CCC), ver);
        self.ccd.describe(&mut pg.nest(params::CCD), ver);
        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
    }
}

impl Describable for HyperSynth {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.hex(params::SCALE, self.scale);
        let dc = &self.default_chord;
        pg.str("CHORD", &format!("{:02X} | {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", dc[0], dc[1], dc[2], dc[3], dc[4], dc[5], dc[6]));
        pg.hex(params::TBLTIC, self.table_tick);
        pg.hex("SHIFT", self.shift);
        pg.hex("SWARM", self.swarm);
        pg.hex("WIDTH", self.width);
        pg.hex("SUBOSC", self.subosc);

        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
    }
}

impl Describable for MacroSynth {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::EQ, self.synth_params.associated_eq);
        pg.hex(params::TBLTIC, self.table_tick);
        pg.enumeration("SHAPE", self.shape as u8, &format!("{:?}", self.shape));
        pg.hex("TIMBRE", self.timbre);
        pg.hex("COLOR", self.color);
        pg.hex("DEGRADE", self.degrade);
        pg.hex("REDUX", self.redux);

        self.synth_params.describe_with_dic(pg, self.filter_types(ver), ver);
        describe_modulators(&self.synth_params, pg, self.destination_names(ver), ver);
    }
}

impl Describable for ControlChange {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, _ver: Version) {
        pg.hex("CC", self.number);
        pg.hex("VAL", self.value);
    }
}

impl Describable for MIDIOut {
    fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        pg.str(params::NAME, &self.name);
        pg.bool(params::TRANSPOSE, self.transpose);
        pg.hex(params::TBLTIC, self.table_tick);

        let port_str = self.human_readable_port();
        pg.enumeration("PORT", self.port, port_str);
        pg.hex("CHANNEL", self.channel);
        pg.hex("BANK", self.bank_select);
        pg.hex("PROGRAM", self.program_change);
        self.custom_cc[0].describe(&mut pg.nest("CCA"), ver);
        self.custom_cc[1].describe(&mut pg.nest("CCB"), ver);
        self.custom_cc[2].describe(&mut pg.nest("CCC"), ver);
        self.custom_cc[3].describe(&mut pg.nest("CCD"), ver);
        self.custom_cc[4].describe(&mut pg.nest("CCE"), ver);
        self.custom_cc[5].describe(&mut pg.nest("CCF"), ver);
        self.custom_cc[6].describe(&mut pg.nest("CCG"), ver);
        self.custom_cc[7].describe(&mut pg.nest("CCH"), ver);
        self.custom_cc[8].describe(&mut pg.nest("CCI"), ver);
        self.custom_cc[9].describe(&mut pg.nest("CCJ"), ver);

        describe_modulators(&self.mods, pg, self.destination_names(ver), ver);
    }
}

impl DescribableWithDictionary for ADSREnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::ATTACK, self.attack);
        pg.hex(params::DECAY, self.decay);
        pg.hex(params::SUSTAIN, self.sustain);
        pg.hex(params::RELEASE, self.release);
    }
}

impl DescribableWithDictionary for AHDEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::ATTACK, self.attack);
        pg.hex(params::HOLD, self.hold);
        pg.hex(params::DECAY, self.decay);
    }
}

impl DescribableWithDictionary for DrumEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::PEAK, self.peak);
        pg.hex(params::BODY, self.body);
        pg.hex(params::DECAY, self.decay);
    }
}

impl DescribableWithDictionary for LFO {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.enumeration(params::LFOSHAPE, self.shape as u8, &format!("{:?}", self.shape));
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::FREQ, self.freq);
        pg.enumeration(params::TRIGGER, self.shape as u8, &format!("{:?}", self.trigger_mode));
    }
}

impl DescribableWithDictionary for TrackingEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::SOURCE, self.src);
        pg.hex("LVAL", self.lval);
        pg.hex("HVAL", self.hval);
    }
}

impl DescribableWithDictionary for TrigEnv {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &[&'static str], _ver: Version) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::ATTACK, self.attack);
        pg.hex(params::HOLD, self.hold);
        pg.str(params::SOURCE, &format!("{:?}", self.src));
    }
}

fn describe_mod<PG : ParameterGatherer>(modulator: &Mod, pg: &mut PG, ix: usize, dests:&[&'static str], ver: Version) {
    let ix = ix + 1;
    match modulator {
        Mod::AHDEnv(ahd)  => {
            pg.enumeration(&format!("MOD{ix}"), 0, "AHD ENV");
            ahd.describe_with_dic(pg, dests, ver);
        },
        Mod::ADSREnv(adsr) => {
            pg.enumeration(&format!("MOD{ix}"), 1, "ADSR ENV");
            adsr.describe_with_dic(pg, dests, ver);
        },
        Mod::DrumEnv(drum_env) =>{
            pg.enumeration(&format!("MOD{ix}"), 1, "DRUM ENV");
            drum_env.describe_with_dic(pg, dests, ver)
        }
        Mod::LFO(lfo) => {
            pg.enumeration(&format!("MOD{ix}"), 1, "LFO");
            lfo.describe_with_dic(pg, dests, ver)
        }
        Mod::TrigEnv(tenv) => {
            pg.enumeration(&format!("MOD{ix}"), 1, "TRIGENV");
            tenv.describe_with_dic(pg, dests, ver);
        }
        Mod::TrackingEnv(tenv) => {
            pg.enumeration(&format!("MOD{ix}"), 1, "TRACKENV");
            tenv.describe_with_dic(pg, dests, ver)
        },
    }
}

pub fn describe_modulators<PG : ParameterGatherer>(sp: &SynthParams, pg: &mut PG, dests: &[&'static str], ver: Version) {
    describe_mod(&sp.mods[0], &mut pg.nest("MOD1"), 0, dests, ver);
    describe_mod(&sp.mods[1], &mut pg.nest("MOD2"), 1, dests, ver);
    describe_mod(&sp.mods[2], &mut pg.nest("MOD3"), 2, dests, ver);
    describe_mod(&sp.mods[3], &mut pg.nest("MOD4"), 3, dests, ver);
}

pub fn describe_succint_params<PG : ParameterGatherer>(sp: &SynthParams, pg: &mut PG, _ver: Version) {
    pg.hex(params::EQ, sp.associated_eq);
    pg.hex(dests::AMP, sp.amp);
    pg.enumeration("LIM", sp.limit.0, sp.limit.str());
    pg.hex(dests::PAN, sp.mixer_pan);
    pg.hex("DRY", sp.mixer_dry);
    pg.hex("CHORUS", sp.mixer_chorus);
    pg.hex("DELAY", sp.mixer_delay);
    pg.hex("REVERB", sp.mixer_reverb);
}

impl DescribableWithDictionary for SynthParams {
    fn describe_with_dic<PG : ParameterGatherer>(&self, pg: &mut PG, filters: &[&str], ver: Version) {
        pg.hex("FINE", self.fine_tune);

        match filters.get(self.filter_type as usize) {
            None =>
                pg.enumeration("FILTER", self.filter_type, &format!("{:02X}", self.filter_type)),
            Some(str) => 
                pg.enumeration("FILTER", self.filter_type, str)
        };

        pg.hex("CUT", self.filter_cutoff);
        pg.hex("RES", self.filter_res);
        describe_succint_params(self, pg, ver);
    }
}
