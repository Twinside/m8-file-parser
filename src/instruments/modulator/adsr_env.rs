use crate::{instruments::ParameterGatherer, params, Version};

use super::{M8Result, Reader, Writer};


#[derive(PartialEq, Debug, Clone)]
pub struct ADSREnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

const ADSRENV_COMMAND_NAMES : [[&'static str; 5]; 4] =
  [
    ["EA1", "AT1", "DE1", "SU1", "ET1"],
    ["EA2", "AT2", "DE2", "SU2", "ET2"],
    ["EA3", "AT3", "DE3", "SU3", "ET3"],
    ["EA4", "AT4", "DE4", "SU4", "ET4"],
  ];

impl ADSREnv {
    pub fn command_name(_ver: Version, mod_id: usize) -> &'static[&'static str] {
        &ADSRENV_COMMAND_NAMES[mod_id]
    }

    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, dests: &'static[&'static str]) {
        let dest_str = dests.get(self.dest as usize).unwrap_or(&"??");
        pg.enumeration(params::DEST, self.dest, dest_str);
        pg.hex(params::AMOUNT, self.amount);
        pg.hex(params::ATTACK, self.attack);
        pg.hex(params::DECAY, self.decay);
        pg.hex(params::SUSTAIN, self.sustain);
        pg.hex(params::RELEASE, self.release);
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.attack);
        w.write(self.decay);
        w.write(self.sustain);
        w.write(self.release);
    }

    pub fn from_reader(reader: &mut Reader, dest: u8) -> M8Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            attack: reader.read(),
            decay: reader.read(),
            sustain: reader.read(),
            release: reader.read(),
        })
    }
}
