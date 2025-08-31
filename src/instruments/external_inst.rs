use array_concat::concat_arrays;

use super::common::SynthParams;
use super::common::TranspEq;
use super::dests;
use super::midi::ControlChange;
use super::params;
use super::CommandPack;
use super::Version;
use crate::reader::*;
use crate::writer::Writer;
use crate::SEND_COMMAND_NAMES;
use crate::SEND_COMMAND_NAMES_6_2;

#[derive(PartialEq, Debug, Clone)]
pub struct ExternalInst {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub input: u8,
    pub port: u8,
    pub channel: u8,
    pub bank: u8,
    pub program: u8,
    pub cca: ControlChange,
    pub ccb: ControlChange,
    pub ccc: ControlChange,
    pub ccd: ControlChange,
}

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const EXTERNAL_INST_BASE_COMMANDS : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT - 3] = [
    "VOL",
    "PIT",
    "MPB",
    "MPG",
    "CCA",
    "CCB",
    "CCC",
    "CCD",
    "FLT",
    "CUT",
    "RES",
    "AMP",
    "LIM",
    "PAN",
    "DRY",
];

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const EXTERNAL_INST_EXTRA_COMMANDS : [&'static str; 2] = [
    // EXTRA
    "ADD",
    "CHD"
];

const EXTERNAL_INST_COMMANDS : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    concat_arrays!(EXTERNAL_INST_BASE_COMMANDS, SEND_COMMAND_NAMES, EXTERNAL_INST_EXTRA_COMMANDS);

const EXTERNAL_INST_COMMANDS_6_2 : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 2] =
    concat_arrays!(EXTERNAL_INST_BASE_COMMANDS, SEND_COMMAND_NAMES_6_2, EXTERNAL_INST_EXTRA_COMMANDS);

#[rustfmt::skip] // Keep constants with important order vertical for maintenance
const DESTINATIONS : [&'static str; 14] = [
    dests::OFF,
    dests::VOLUME,
    dests::CUTOFF,
    dests::RES,
    dests::AMP,
    dests::PAN,
    params::CCA,
    params::CCB,
    params::CCC,
    params::CCD,
    dests::MOD_AMT,
    dests::MOD_RATE,
    dests::MOD_BOTH,
    dests::MOD_BINV,
];

impl ExternalInst {
    const MOD_OFFSET: usize = 22;

    pub fn command_name(&self, ver: Version) -> &'static [&'static str] {
        if ver.at_least(6, 1) {
            &EXTERNAL_INST_COMMANDS_6_2
        } else {
            &EXTERNAL_INST_COMMANDS
        }
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    /// List of all the applyable filter types for the instrument
    pub fn filter_types(&self, _ver: Version) -> &'static [&'static str] {
        &super::common::COMMON_FILTER_TYPES
    }

    /// Return human readable name of the port.
    pub fn human_readable_port(&self) -> &'static str {
        crate::instruments::midi::PORTS[self.port as usize]
    }

    pub fn write(&self, ver: Version, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(TranspEq::from(ver, self.transpose, self.synth_params.associated_eq).into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.input);
        w.write(self.port);
        w.write(self.channel);
        w.write(self.bank);
        w.write(self.program);

        self.cca.write(w);
        self.ccb.write(w);
        self.ccc.write(w);
        self.ccd.write(w);

        self.synth_params.write(ver, w, ExternalInst::MOD_OFFSET);
    }

    pub fn from_reader(ver: Version, reader: &mut Reader, number: u8) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = TranspEq::from_version(ver, reader.read());

        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let input = reader.read();
        let port = reader.read();
        let channel = reader.read();
        let bank = reader.read();
        let program = reader.read();
        let cca = ControlChange::from_reader(reader)?;
        let ccb = ControlChange::from_reader(reader)?;
        let ccc = ControlChange::from_reader(reader)?;
        let ccd = ControlChange::from_reader(reader)?;

        let synth_params = SynthParams::from_reader3(
            ver,
            reader,
            volume,
            pitch,
            fine_tune,
            transp_eq.eq,
            ExternalInst::MOD_OFFSET,
        )?;

        Ok(ExternalInst {
            number,
            name,
            transpose: transp_eq.transpose,
            table_tick,
            synth_params,

            input,
            port,
            channel,
            bank,
            program,
            cca,
            ccb,
            ccc,
            ccd,
        })
    }
}
