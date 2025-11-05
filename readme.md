# m8-file-parser

[![Crates.io](https://img.shields.io/crates/v/m8-file-parser)](https://crates.io/crates/m8-file-parser)
[![Docs.rs](https://docs.rs/m8-file-parser/badge.svg)](https://docs.rs/m8-file-parser)

Reads [Dirtwave M8](https://dirtywave.com/) files into Rust structs. Covers M8 firmware version 4.0 to 6.0

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work.

Big thanks to AlexCharlton for the original version of the package [m8-files](https://github.com/AlexCharlton/m8-files).

## Usage

Add to your `Cargo.toml`:
```
m8-file-parser = "0.6"
```
Or
```
$ cargo add m8-file-parser
```


Load an example song:
```
$ cargo run --example read_song -- examples/songs/DEFAULT.m8s
```

## TODO

- Add song groove, scale, note_preview
- Add settings: output/speaker volume
- Displays: MixerSettings, EffectsSettings, MidiSettings, MidiMapping
