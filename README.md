Gameboy Emulator
================

## Compiling and Running

Requires Rust: https://rustup.rs/

Compile:
```
$ cargo build --release
```

Run:
```
$ ./target/release/gameboy_emulator ROM_PATH
```

Alternatively you can directly run with cargo:
```
$ cargo run --release -- ROM_PATH
```

Use `--help` to see more options
```
$ ./target/release/gameboy_emulator --help
```

## Joypad Mapping

| Gameboy | Emulator       |
|---------|----------------|
| Arrows  | Arrows or ZQSD |
| A       | O              |
| B       | P              |
| Start   | Enter/Return   |
| Select  | Left Control   |


## Still missing

- The sound controller is currently not implemented
- Only MBC1 (and only the banking mode 0) is currently implemented
- The PPU implementation uses a fetcher and a Pixel FIFO but is not timing accurate (the CPU should be in the other hand)
- A lot of bugs are *not* implemented, like the Halt-bug or the OAM-bug
- In the actual Gameboy, the VRAM access is disabled during some PPU modes. This is not implemented