# m8-file-parser

Reads [Dirtwave M8](https://dirtywave.com/) files into Rust structs.

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work.

Big thanks to AlexCharlton for the original version of the package [m8-files](https://github.com/AlexCharlton/m8-files).

## Usage

Add to your `Cargo.toml`:
```
m8-file-parser = "0.4"
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
