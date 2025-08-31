use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{reader::*, Version};

#[derive(PartialEq, Debug, Clone)]
pub struct MidiSettings {
    pub receive_sync: bool,
    pub receive_transport: u8,
    pub send_sync: bool,
    pub send_transport: u8,
    pub record_note_channel: u8,
    pub record_note_velocity: bool,
    pub record_note_delay_kill_commands: u8,
    pub control_map_channel: u8,
    pub song_row_cue_channel: u8,
    pub track_input_channel: [u8; 8],
    pub track_input_intrument: [u8; 8],
    pub track_input_program_change: bool,
    pub track_input_mode: u8,
}

impl TryFrom<&mut Reader> for MidiSettings {
    type Error = ParseError;

    fn try_from(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            receive_sync: reader.read_bool(),
            receive_transport: reader.read(),
            send_sync: reader.read_bool(),
            send_transport: reader.read(),
            record_note_channel: reader.read(),
            record_note_velocity: reader.read_bool(),
            record_note_delay_kill_commands: reader.read(),
            control_map_channel: reader.read(),
            song_row_cue_channel: reader.read(),
            track_input_channel: reader.read_bytes(8).try_into().unwrap(),
            track_input_intrument: reader.read_bytes(8).try_into().unwrap(),
            track_input_program_change: reader.read_bool(),
            track_input_mode: reader.read(),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct LimiterParameter {
    pub level: u8,
    pub attack_release: Option<(u8, u8, bool)>
}

#[derive(PartialEq, Debug, Clone)]
pub struct MixerSettings {
    pub master_volume: u8,
    pub track_volume: [u8; 8],
    pub chorus_volume: u8,
    pub delay_volume: u8,
    pub reverb_volume: u8,
    pub analog_input: AnalogInputSettings,
    pub usb_input: InputMixerSettings,
    pub dj_filter: u8,
    pub dj_peak: u8,
    pub dj_filter_type: u8,
    pub limiter: LimiterParameter,
    pub ott_level: Option<u8>
}

impl MixerSettings {
    pub(crate) fn from_reader(reader: &mut Reader, ver: Version) -> M8Result<Self> {
        let master_volume = reader.read();
        let master_limit = reader.read();
        let track_volume: [u8; 8] = reader.read_bytes(8).try_into().unwrap();
        let chorus_volume = reader.read();
        let delay_volume = reader.read();
        let reverb_volume = reader.read();
        let analog_input_volume = (reader.read(), reader.read());
        let usb_input_volume = reader.read();

        let analog_input_l =
            InputMixerSettings::from_reader(reader, analog_input_volume.0);
        let analog_input_r =
            InputMixerSettings::from_reader(reader, analog_input_volume.0);
        let usb_input_chorus = reader.read();
        let usb_input_delay = reader.read();
        let usb_input_reverb = reader.read();

        let analog_input = if analog_input_volume.1 == 255 {
            AnalogInputSettings::Stereo(analog_input_l)
        } else {
            AnalogInputSettings::DualMono((analog_input_l, analog_input_r))
        };
        let usb_input = InputMixerSettings {
            volume: usb_input_volume,
            mfx: usb_input_chorus,
            delay: usb_input_delay,
            reverb: usb_input_reverb,
        };

        let dj_filter = reader.read();
        let dj_peak = reader.read();
        let dj_filter_type = reader.read();

        let limiter_conf = if !ver.at_least(6, 0) {
            None
        } else {
            let limiter_attack = reader.read();
            let limiter_release = reader.read();
            let soft_clip = reader.read();
            Some((limiter_attack, limiter_release, soft_clip != 0))
        };

        let ott_level = if ver.at_least(6, 1) {
            Some(reader.read())
        } else {
            None
        };

        Ok(Self {
            master_volume,
            track_volume,
            chorus_volume,
            delay_volume,
            reverb_volume,
            analog_input,
            usb_input,
            dj_filter,
            dj_peak,
            dj_filter_type,
            limiter: LimiterParameter {
                level: master_limit,
                attack_release: limiter_conf 
            },
            ott_level
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct InputMixerSettings {
    pub volume: u8,
    pub mfx: u8,
    pub delay: u8,
    pub reverb: u8,
}

impl InputMixerSettings {
    pub fn from_reader(reader: &mut Reader, volume: u8) -> Self {
        let chorus = reader.read();
        let delay = reader.read();
        let reverb = reader.read();

        Self {
            volume, mfx: chorus, delay, reverb
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AnalogInputSettings {
    Stereo(InputMixerSettings),
    DualMono((InputMixerSettings, InputMixerSettings)),
}

/// Effect filter configuration only used in old versions
/// of the firmware before being replaced with EQ
#[derive(PartialEq, Debug, Clone)]
pub struct EffectFilter {
    pub high_pass: u8,
    pub low_pass: u8
}

impl EffectFilter {
    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        let high_pass = reader.read();
        let low_pass = reader.read();
        Ok(EffectFilter { high_pass, low_pass })
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive, PartialEq, Copy, Clone, Default, Debug)]
pub enum FxKind {
    #[default]
    Chorus,
    Phaser,
    Flanger
}

#[derive(PartialEq, Debug, Clone)]
pub struct OttConfiguration {
    pub time: u8,
    pub color: u8
}

impl OttConfiguration {
    pub(crate) fn from_reader(reader: &mut Reader, _version: Version) -> M8Result<Self> {
        Ok(Self {
            time: reader.read(),
            color: reader.read()
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct EffectsSettings {
    pub mfx_kind : Option<FxKind>,
    pub chorus_mod_depth: u8,
    pub chorus_mod_freq: u8,
    pub chorus_width: u8,
    pub chorus_reverb_send: u8,

    pub delay_filter: Option<EffectFilter>,
    pub delay_time_l: u8,
    pub delay_time_r: u8,
    pub delay_feedback: u8,
    pub delay_width: u8,
    pub delay_reverb_send: u8,

    pub reverb_filter: Option<EffectFilter>,
    pub reverb_size: u8,
    pub reverb_damping: u8,
    pub reverb_mod_depth: u8,
    pub reverb_mod_freq: u8,
    pub reverb_width: u8,
    pub reverb_shimmer: Option<u8>,
    pub ott_configuration: Option<OttConfiguration>
}

impl EffectsSettings {
    pub(crate) fn from_reader(reader: &mut Reader, version: Version) -> M8Result<Self> {
        let chorus_mod_depth = reader.read();
        let chorus_mod_freq = reader.read();
        let chorus_width = reader.read();
        let chorus_reverb_send = reader.read();
        reader.read_bytes(3); //unused

        let delay_filter=  EffectFilter::from_reader(reader)?;
        let delay_filter = if version.at_least(4, 0) {
            None
        } else {
            Some(delay_filter)
        };

        let delay_time_l = reader.read();
        let delay_time_r = reader.read();
        let delay_feedback = reader.read();
        let delay_width = reader.read();
        let delay_reverb_send = reader.read();
        reader.read_bytes(1); //unused

        let reverb_filter= EffectFilter::from_reader(reader)?;
        let reverb_filter = if version.at_least(4, 0) {
            None
        } else {
            Some(reverb_filter)
        };

        let reverb_size = reader.read();
        let reverb_damping = reader.read();
        let reverb_mod_depth = reader.read();
        let reverb_mod_freq = reader.read();
        let reverb_width = reader.read();
        let (reverb_shimmer, ott_configuration, mfx_kind) =
            if version.at_least(6, 1) {
                let shimmer = Some(reader.read());
                let ott = OttConfiguration::from_reader(reader, version)?;
                let mfx = reader.read();
                let kind =
                   mfx.try_into().map_err(| _| ParseError(format!("Unknown MFX kind {}", mfx)))?;
                (shimmer, Some(ott), Some(kind))

            } else {
                (None, None, None)
            };

        Ok(Self {
            mfx_kind,
            chorus_mod_depth,
            chorus_mod_freq,
            chorus_width,
            chorus_reverb_send,

            delay_filter,
            delay_time_l,
            delay_time_r,
            delay_feedback,
            delay_width,
            delay_reverb_send,

            reverb_filter,
            reverb_size,
            reverb_damping,
            reverb_mod_depth,
            reverb_mod_freq,
            reverb_width,
            reverb_shimmer,
            ott_configuration
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MidiMapping {
    pub channel: u8,
    pub control_number: u8,
    pub value: u8,
    pub typ: u8,
    pub param_index: u8,
    pub min_value: u8,
    pub max_value: u8,
}

impl MidiMapping {
    pub(crate) fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            channel: reader.read(),
            control_number: reader.read(),
            value: reader.read(),
            typ: reader.read(),
            param_index: reader.read(),
            min_value: reader.read(),
            max_value: reader.read(),
        })
    }

    pub fn empty(&self) -> bool {
        self.channel == 0
    }
}
