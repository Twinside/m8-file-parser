# m8-files

## Original crate info

This does not represent _this_ repository, but here they are:

[![Crates.io](https://img.shields.io/crates/v/m8-files)](https://crates.io/crates/m8-files)
[![Docs.rs](https://docs.rs/m8-files/badge.svg)](https://docs.rs/m8-files)

Reads [Dirtwave M8](https://dirtywave.com/) files into Rust structs.

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work and [m8-files (main)](https://github.com/AlexCharlton/m8-files)
that the initial conversion.

## Usage

For usage of the original library published on crates.io, see [m8-files (main)](https://github.com/AlexCharlton/m8-files)

## TODO

- Add song groove, scale, note_preview
- Add settings: output/speaker volume
- Displays: MixerSettings, MidiSettings, MidiMapping

## Changelog

### 0.3

- v4 reading
- v4 overwriting, you can load, modify elements and rewrite the same song.
  * Does not work with song other than v4/v4.1
- Added EQ with plotting
- Mapped all FX instruction depending on the names
- Mapped enums to many instrument parameters with human readable information

### 0.2
- Add V3 support
- Fix instrument alignment issues
