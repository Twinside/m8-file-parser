use std::f64::consts::PI;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{reader::*, ParameterGatherer, Version};

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum EqType {
    #[default]
    LowCut = 0,
    LowShelf = 1,
    Bell = 2,
    BandPass = 3,
    HiShelf = 4,
    HiCut = 5
}

const EQ_TYPE_STR : [&'static str; 6] =
    [
        "LOWCUT",
        "LOWSHELF",
        "BELL",
        "BANDPASS",
        "HI.SHELF",
        "HI.CUT"
    ];

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum EqMode {
    #[default]
    Stereo = 0,
    Mid = 1,
    Side = 2,
    Left = 3,
    Right = 4
}

const EQ_MODE_STR : [&'static str; 5] =
    [
        "STEREO",
        "MID",
        "SIDE",
        "LEFT",
        "RIGHT"
    ];

#[derive(PartialEq, Eq, Clone, Debug, Copy, Default)]
pub struct EqModeType(pub u8);

impl EqModeType {
    pub fn new(ty: EqType, mode : EqMode) -> EqModeType {
        EqModeType(ty as u8 | ((mode as u8) << 5))
    }

    pub fn eq_mode_hex(&self) -> u8 {
        (self.0 >> 5)& 0x7
    }

    pub fn eq_type(&self) -> EqType {
        EqType::try_from(self.eq_type_hex())
            .unwrap_or(EqType::Bell)
    }

    pub fn eq_type_hex(&self) -> u8 { self.0 & 0x7 }

    pub fn eq_mode(&self) -> EqMode {
        EqMode::try_from(self.eq_mode_hex())
            .unwrap_or(EqMode::Stereo)
    }

    pub fn mode_str(&self) -> &'static str {
        let index = self.eq_mode_hex() as usize;
        EQ_MODE_STR.get(index).unwrap_or(&"")
    }

    pub fn type_str(&self) -> &'static str {
        let index = self.eq_type_hex() as usize;
        EQ_TYPE_STR.get(index).unwrap_or(&"")
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct EqBand {
    pub mode      : EqModeType,

    pub freq_fin  : u8,
    pub freq      : u8,

    pub level_fin : u8,
    pub level     : u8,

    pub q         : u8
}

/// See https://www.w3.org/TR/audio-eq-cookbook/
#[derive(Debug)]
pub struct BiQuadCoeffs {
    pub a0: f64,
    pub a1: f64,
    pub a2: f64,

    pub b0: f64,
    pub b1: f64,
    pub b2: f64
}

pub struct BiQuadFreqResponseCoeff {
    pub cst_num: f64,
    pub cstp_num : f64,
    pub coeff_num: f64,

    pub cst_denom: f64,
    pub cstp_denom : f64,
    pub coeff_denom: f64
}

impl BiQuadFreqResponseCoeff {
    pub fn response(&self, p : f64) -> f64 {
        let num = self.cst_num - p * (self.coeff_num * (1.0 - p) + self.cstp_num);
        let denom = self.cst_denom - p * (self.coeff_denom * (1.0 - p) + self.cstp_denom);
        (num / denom).sqrt()
    }
}

impl BiQuadCoeffs {
    pub fn freq_response_coeff(&self) -> BiQuadFreqResponseCoeff {
        BiQuadFreqResponseCoeff {
            cst_num: ((self.b0 + self.b1 + self.b2) / 2.0).powi(2),
            cstp_num : (self.b1 * (self.b0 + self.b2)),
            coeff_num: 4.0 * self.b0 * self.b2,

            cst_denom: ((self.a0 + self.a1 + self.a2) / 2.0).powi(2),
            cstp_denom : (self.a1 * (self.a0 + self.a2)),
            coeff_denom: 4.0 * self.a0 * self.a2
        }
    }

    pub fn normalized(&self) -> Self {
        let a0 = self.a0;
        Self {
            a0: 1.0,
            a1: self.a1 / a0,
            a2: self.a2 / a0,
            b0: self.b0 / a0,
            b1: self.b1 / a0,
            b2: self.b2 / a0
        }
    }
}

impl EqBand {
    const V4_SIZE : usize = 6;

    pub fn default_low() -> EqBand {
        let freq = 100 as usize;
        EqBand {
            mode: EqModeType::new(EqType::LowShelf, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }

    pub fn default_mid() -> EqBand {
        let freq = 1000 as usize;
        EqBand {
            mode: EqModeType::new(EqType::Bell, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }

    pub fn default_high() -> EqBand {
        let freq = 5000 as usize;
        EqBand {
            mode: EqModeType::new(EqType::HiShelf, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }


    pub fn is_empty(&self) -> bool {
        self.level == 0 && self.level_fin == 0
    }

    /// See https://www.w3.org/TR/audio-eq-cookbook/
    pub fn coeffs(&self, sample_rate: usize) -> BiQuadCoeffs {
        let sample_rate = sample_rate as f64;
        let a = (10.0 as f64).powf(self.gain()/40.0);
        let q = 
            if self.mode.eq_type() == EqType::Bell {
                (self.q as f64) * a
            } else {
                (self.q as f64) / 100.0
            };

        let w0 = 2.0 * PI * (self.frequency() as f64) / sample_rate;
        let alpha = w0.sin() / (2.0 * q);

/*

notch:      H(s) = (s^2 + 1) / (s^2 + s/Q + 1)

            b0 =   1
            b1 =  -2*cos(w0)
            b2 =   1
            a0 =   1 + alpha
            a1 =  -2*cos(w0)
            a2 =   1 - alpha



APF:        H(s) = (s^2 - s/Q + 1) / (s^2 + s/Q + 1)

            b0 =   1 - alpha
            b1 =  -2*cos(w0)
            b2 =   1 + alpha
            a0 =   1 + alpha
            a1 =  -2*cos(w0)
            a2 =   1 - alpha



 */
        let cw0 = w0.cos();
        match self.mode.eq_type() {
            EqType::LowCut => {
                // HPF:        H(s) = s^2 / (s^2 + s/Q + 1)
                BiQuadCoeffs {
                    b0:  (1.0 + cw0)/2.0,
                    b1: -(1.0 + cw0),
                    b2:  (1.0 + cw0)/2.0,
                    a0:   1.0 + alpha,
                    a1:  -2.0*cw0,
                    a2:   1.0 - alpha
                }
            }
            EqType::LowShelf => {
                // lowShelf: H(s) = A * (s^2 + (sqrt(A)/Q)*s + A)/(A*s^2 + (sqrt(A)/Q)*s + 1)

                let sqra = a.sqrt();
                BiQuadCoeffs {
                    b0:      a*( (a+1.0) - (a-1.0)*cw0 + 2.0 * sqra * alpha ),
                    b1:  2.0*a*( (a-1.0) - (a+1.0)*cw0                   ),
                    b2:      a*( (a+1.0) - (a-1.0)*cw0 - 2.0 * sqra*alpha ),
                    a0:        (a+1.0) + (a-1.0)*cw0 + 2.0 * sqra*alpha,
                    a1:   -2.0*( (a-1.0) + (a+1.0)*cw0                   ),
                    a2:        (a+1.0) + (a-1.0)*cw0 - 2.0 *sqra*alpha
                }
            },
            EqType::Bell => {
                // peakingEQ:  H(s) = (s^2 + s*(A/Q) + 1) / (s^2 + s/(A*Q) + 1)
                BiQuadCoeffs {
                    b0: 1.0 + alpha * a,
                    b1: -2.0 * cw0,
                    b2: 1.0 - alpha * a,
                    a0: 1.0 + alpha / a,
                    a1: -2.0 * cw0,
                    a2: 1.0 - alpha / a
                }
            },
            EqType::BandPass => {
                // BPF:        H(s) = (s/Q) / (s^2 + s/Q + 1)      (constant 0 dB peak gain)
                BiQuadCoeffs {
                    b0:   alpha,
                    b1:   0.0,
                    b2:  -alpha,
                    a0:   1.0 + alpha,
                    a1:  -2.0*cw0,
                    a2:   1.0 - alpha,
                }
            },
            EqType::HiShelf => {
                // highShelf: H(s) = A * (A*s^2 + (sqrt(A)/Q)*s + 1)/(s^2 + (sqrt(A)/Q)*s + A)
                let sqrta = a.sqrt();
                BiQuadCoeffs {
                    b0:      a*( (a+1.0) + (a-1.0)*cw0 + 2.0*sqrta*alpha ),
                    b1: -2.0*a*( (a-1.0) + (a+1.0)*cw0                   ),
                    b2:      a*( (a+1.0) + (a-1.0)*cw0 - 2.0*sqrta*alpha ),
                    a0:        (a+1.0) - (a-1.0)*cw0 + 2.0*sqrta*alpha,
                    a1:    2.0*( (a-1.0) - (a+1.0)*cw0                   ),
                    a2:        (a+1.0) - (a-1.0)*cw0 - 2.0*sqrta*alpha
                }
            },
            EqType::HiCut => {
                // LPF:        H(s) = 1 / (s^2 + s/Q + 1)
                BiQuadCoeffs {
                    b0:  (1.0 - cw0)/2.0,
                    b1:   1.0 - cw0,
                    b2:  (1.0 - cw0)/2.0,
                    a0:   1.0 + alpha,
                    a1:  -2.0*cw0,
                    a2:   1.0 - alpha
                }
            }
        }
    }

    pub fn gain(&self) -> f64 {
        let int_gain = ((self.level as i16) << 8) | (self.level_fin as i16);
        (int_gain as f64) / 100.0
    }

    pub fn frequency(&self) -> usize {
        ((self.freq as usize) << 8) | self.freq_fin as usize
    }

    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, _ver: Version) {
        pg.float("GAIN", self.gain());
        pg.float("FREQ", self.frequency() as f64);
        pg.hex("Q", self.q);
        pg.enumeration("TYPE", self.mode.eq_type_hex(), self.mode.type_str());
        pg.enumeration("MODE", self.mode.eq_mode_hex(), self.mode.mode_str());
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.mode.0);
        w.write(self.freq_fin);
        w.write(self.freq);
        w.write(self.level_fin);
        w.write(self.level);
        w.write(self.q);
    }

    pub fn from_reader(reader: &mut Reader) -> EqBand {
        let mode = EqModeType(reader.read());
        let freq_fin = reader.read();
        let freq = reader.read();
        let level_fin = reader.read();
        let level = reader.read();
        let q = reader.read();

        Self { level, level_fin, freq, freq_fin, mode, q }
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct Equ {
    pub low : EqBand,
    pub mid : EqBand,
    pub high : EqBand
}

impl Equ {
    pub const V4_SIZE : usize = 3 * EqBand::V4_SIZE;

    pub fn is_empty(&self) -> bool {
        self.low.is_empty() && self.mid.is_empty() && self.high.is_empty()
    }

    pub fn clear(&mut self) {
        self.low = EqBand::default_low();
        self.mid = EqBand::default_mid();
        self.high = EqBand::default_high();
    }

    pub fn accumulate(&self, freqs: &[f64], mode: EqMode) -> Vec<f64> {
        let sample_rate = 44100;
        let mut coeffs = vec![];

        if self.low.mode.eq_mode() == mode {
            let c0 = self.low.coeffs(sample_rate).normalized().freq_response_coeff();
            coeffs.push(c0);
        }

        if self.mid.mode.eq_mode() == mode {
            let c1 = self.mid.coeffs(sample_rate).normalized().freq_response_coeff();
            coeffs.push(c1);
        }

        if self.high.mode.eq_mode() == mode {
            let c2 = self.high.coeffs(sample_rate).normalized().freq_response_coeff();
            coeffs.push(c2);
        }

        freqs
          .iter()
          .map(|freq| {
              let mut p = ((PI * freq) / (sample_rate as f64)).sin();
              p = p * p;
         
              let mut v = 1.0;

              for c in &coeffs {
                v *= c.response(p)
              }

              v.log10()
          }).collect()
    }

    pub fn describe<PG : ParameterGatherer>(&self, pg: &mut PG, ver: Version) {
        self.low.describe(&mut pg.nest("LOW"), ver);
        self.mid.describe(&mut pg.nest("MID"), ver);
        self.high.describe(&mut pg.nest("HIGH"), ver);
    }

    pub fn write(&self, w: &mut Writer) {
        self.low.write(w);
        self.mid.write(w);
        self.high.write(w);
    }

    pub fn from_reader(reader: &mut Reader) -> Equ {
        let low = EqBand::from_reader(reader);
        let mid = EqBand::from_reader(reader);
        let high = EqBand::from_reader(reader);
        Self { low, mid, high }
    }
}

pub const CHORUS_EQ_IDX : usize = 33;
pub const DELAY_EQ_IDX : usize = 34;
pub const REVERB_EQ_IDX : usize = 35;
