use crate::{reader::*, writer::Writer};

use std::fmt;

#[derive(PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 4,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

pub const FIRMWARE_3_0_SONG_VERSION : Version =
    Version { major: 3, minor: 0, patch: 0 };

/// eq introduction
pub const FIRMWARE_4_0_SONG_VERSION : Version =
    Version { major: 4, minor: 0, patch: 0 };

/// 128 eq support
pub const FIRMWARE_5_0_SONG_VERSION : Version =
    Version { major: 4, minor: 1, patch: 0 };

pub const FIRMWARE_6_0_SONG_VERSION : Version =
    Version { major: 6, minor: 0, patch: 0 };

pub const FIRMWARE_6_2_SONG_VERSION : Version =
    Version { major: 6, minor: 1, patch: 0 };

impl Version {
    pub const SIZE: usize = 14;

    pub fn new(major: u8, minor: u8) -> Version {
        Version { major, minor, patch: 0 }
    }

    pub fn write(&self, w: &mut Writer) {
        w.write_string("M8VERSION", 10);

        w.write((self.minor << 4) | self.patch);
        w.write(self.major);

        w.write(0);
        w.write(0x10); // why? don't know, but borked result if not written
    }

    pub fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        let _version_string = reader.read_bytes(10);
        let lsb = reader.read();
        let msb = reader.read();
        let major = msb & 0x0F;
        let minor = (lsb >> 4) & 0x0F;
        let patch = lsb & 0x0F;

        reader.read_bytes(2); // Skip
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn after(&self, other: &Version) -> bool {
        self.at_least(other.major, other.minor)
    }

    pub fn at_least(&self, major: u8, minor: u8) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}
