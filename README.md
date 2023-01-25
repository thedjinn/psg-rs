# PSG - AY-3-8910 and YM2149 emulation in Rust

[![Crates.io](https://img.shields.io/crates/v/psg.svg)](https://crates.io/crates/psg)
[![Docs.rs](https://docs.rs/psg/badge.svg)](https://docs.rs/psg)
![GitHub](https://img.shields.io/github/license/thedjinn/psg-rs)

The PSG crate provides a fast and highly precise emulation of the [General Instruments AY-3-8910](https://en.wikipedia.org/wiki/General_Instrument_AY-3-8910) Programmable Sound Generator chip, as well as its most popular clone, the Yamaha YM2149.

These PSG chips were used in some of the most popular home computers in the 1980s and early 1990s, such as the MSX family, the Sinclair ZX Spectrum, and the Atari ST.

This particular implementation of the PSG chip was specifically built for use in emulation and music production (e.g. tracker) software, and comes with many useful extras to aid in writing such applications. Examples of these are math functions for easy period/frequency conversion, conversion to/from MIDI note numbers, and APIs for directly setting register values, as well as APIs that expose every individual property of the PSG chip.

## Usage

To start using the PSG emulator in your own projects, add the following line to your Cargo dependencies:

```toml
psg = "1.0.0"
```

After that, simply initialize a new [`PSG`](https://docs.rs/psg/latest/psg/struct.PSG.html) struct, set some registers, and start rendering in a loop:

```rust
// Initialize a new PSG with a clock rate of an MSX machine and a sampling rate of 44100 Hz.
let mut psg = PSG::new(1789772.5, 44100)?;

// Set some registers.
let channel = psg.channel_mut(0);
channel.set_period(100);
channel.set_amplitude(8);
channel.set_tone_disabled(false);

// Render a second of audio.
for _ in 0..44100 {
    let (left, right) = psg.render();

    // Do something useful with the samples here, such as writing to a file or playing on an
    // audio device.
}
```

For more detailed information on how to use the crate, please have a look at the documentation of the [`PSG`](https://docs.rs/psg/latest/psg/struct.PSG.html) struct, which is the workhorse of the crate.

## Acknowledgements

The crate is based on the excellent Ayumi by Peter Sovietov and includes several bug-fixes and improvements, while still being sample-accurate when compared to the original implementation.

## License

As with the original Ayumi implementation, the emulator and all other code in this crate is released under the MIT license.
