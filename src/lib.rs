//! The PSG crate provides a fast and highly precise emulation of the [General Instruments
//! AY-3-8910](https://en.wikipedia.org/wiki/General_Instrument_AY-3-8910) Programmable Sound
//! Generator chip, as well as its most popular clone, the Yamaha YM2149.
//!
//! These PSG chips were used in some of the most popular home computers in the 1980s and early
//! 1990s, such as the MSX family, the Sinclair ZX Spectrum, and the Atari ST.
//!
//! This particular implementation of the PSG chip was specifically built for use in emulation and
//! music production (e.g. tracker) software, and comes with many useful extras to aid in writing
//! such applications. Examples of these are math functions for easy period/frequency conversion,
//! conversion to/from MIDI note numbers, and APIs for directly setting register values, as well as
//! APIs that expose every individual property of the PSG chip.
//!
//! The crate is based on the excellent Ayumi by Peter Sovietov and includes several bug-fixes and
//! improvements, while still being sample-accurate when compared to the original implementation.
//!
//! To get started, simply initialize a new [`PSG`] struct, set some registers, and start rendering in
//! a loop:
//!
//! ```
//! # use psg::PSG;
//! // Initialize a new PSG with a clock rate of an MSX machine and a sampling rate of 44100 Hz.
//! let mut psg = PSG::new(1789772.5, 44100)?;
//!
//! // Set some registers.
//! let channel = psg.channel_mut(0);
//! channel.set_period(100);
//! channel.set_amplitude(8);
//! channel.set_tone_disabled(false);
//!
//! // Render a second of audio.
//! for _ in 0..44100 {
//!     let (left, right) = psg.render();
//!
//!     // Do something useful with the samples here, such as writing to a file or playing on an
//!     // audio device.
//! }
//! # Ok::<(), psg::Error>(())
//! ```
//!
//! For more detailed information on how to use the crate, please have a look at the [`PSG`]
//! struct, which is the workhorse of the crate.

mod channel;
mod dc_filter;
mod decimator;
mod envelope_generator;
mod error;
mod interpolator;
mod noise_generator;

pub mod math;

pub use channel::Channel;
pub use envelope_generator::EnvelopeGenerator;
pub use error::Error;
pub use noise_generator::NoiseGenerator;

use decimator::{DECIMATE_FACTOR, Decimator, FIR_SIZE};
use dc_filter::DCFilter;
use interpolator::Interpolator;

/// Digital-to-analog amplitude conversion table for the AY-3-8910. Internally, amplitudes are
/// represented as 5-bit values. The AY only has 16 amplitude levels. This table therefore contains
/// quantized values.
const AY_DAC_TABLE: [f64; 32] = [
    0.0,             0.0,             0.00999465934234, 0.00999465934234,
    0.0144502937362, 0.0144502937362, 0.0210574502174,  0.0210574502174,
    0.0307011520562, 0.0307011520562, 0.0455481803616,  0.0455481803616,
    0.0644998855573, 0.0644998855573, 0.107362478065,   0.107362478065,
    0.126588845655,  0.126588845655,  0.20498970016,    0.20498970016,
    0.292210269322,  0.292210269322,  0.372838941024,   0.372838941024,
    0.492530708782,  0.492530708782,  0.635324635691,   0.635324635691,
    0.805584802014,  0.805584802014,  1.0,              1.0
];

/// Digital-to-analog amplitude conversion table for the YM2149, utilizing the full 5-bit dynamic
/// range. Note that the PSG registers only support setting the amplitude as a 4-bit value, and
/// that a value of 0 always represents a mute channel. Only the envelope generator uses 5-bit
/// amplitudes.
const YM_DAC_TABLE: [f64; 32] = [
    0.0,             0.0,             0.00465400167849, 0.00772106507973,
    0.0109559777218, 0.0139620050355, 0.0169985503929,  0.0200198367285,
    0.024368657969,  0.029694056611,  0.0350652323186,  0.0403906309606,
    0.0485389486534, 0.0583352407111, 0.0680552376593,  0.0777752346075,
    0.0925154497597, 0.111085679408,  0.129747463188,   0.148485542077,
    0.17666895552,   0.211551079576,  0.246387426566,   0.281101701381,
    0.333730067903,  0.400427252613,  0.467383840696,   0.53443198291,
    0.635172045472,  0.75800717174,   0.879926756695,   1.0
];

/// An enumeration of the various chip variants supported by the PSG struct.
pub enum ChipType {
    /// The original General Instrument AY-3-8910.
    AY,

    /// The Yamaha YM2149. In all respects identical to the AY-3-8910, except for the envelope
    /// generator, which has double the resolution in its digital-to-analog converter, resulting in
    /// smoother envelopes.
    YM
}

impl ChipType {
    /// Return a reference to the digital-to-analog amplitude conversion table for the current chip
    /// type.
    fn log2lin_table(&self) -> &'static [f64; 32] {
        match self {
            ChipType::AY => &AY_DAC_TABLE,
            ChipType::YM => &YM_DAC_TABLE
        }
    }
}

/// The programmable sound generator (PSG). This struct is the workhorse of the crate and
/// contains all state to fully emulate the selected chip, which can either be the original General
/// Instrument AY-3-8912 or the Yamaha YM2149.
///
/// To get a proper audio signal, instantiate the struct with a sample rate of your choice, and a
/// suitable chip clock rate. Here are some common clock rates:
///
///  - Amstrad CPC: 1 MHz
///  - Atari ST: 2 MHz
///  - MSX: 1.7897725 MHz
///  - Oric-1: 1 MHz
///  - ZX Spectrum: 1.7734 MHz
///
/// The base period unit used by the tone generators is the period of a clock cycle multiplied by
/// 16 (the PSG contains a 16x frequency divider). Therefore, with a clock rate of 1 MHz tones with
/// frequencies between 15.26 Hz and 31.25 kHz can be obtained, although in practice this would be
/// limited by half the sampling frequency (the Nyquist frequency). Setting tone frequencies beyond
/// this limit will produce aliasing and is not recommended.
///
/// The noise and envelope periods works similar, but instead of oscillating these components will
/// produce a new value after every period.
///
/// Note that there are envelope shapes that have a repeating pattern (sawtooth and triangle
/// waveforms) and that it is possible to set the envelope period to such a low value that its
/// frequency falls into the audible range. This is the so-called "buzzer" effect and can be used
/// to create timbres that vastly differ from the usual square wave and noise sounds. The effect
/// works best when using the YM2149 chip type, as it has double the dynamic range in the envelope
/// generator.
pub struct PSG {
    channels: [Channel; 3],
    noise_generator: NoiseGenerator,
    envelope_generator: EnvelopeGenerator,

    log2lin_table: &'static [f64; 32],

    // Clock signal
    x: f64,
    step: f64,

    // Interpolators
    left_interpolator: Interpolator,
    right_interpolator: Interpolator,

    // Decimators (anti-alias filters)
    left_decimator: Decimator,
    right_decimator: Decimator,
    decimator_index: usize,

    // DC filter
    dc_filter: DCFilter
}

impl PSG {
    /// Initialize a new PSG struct using the specified clock and sample rates.
    ///
    /// There is an upper bound to the clock rate that can be used for a given sample rate. This
    /// upper limit can be computed by multiplying the sample rate by 128. Providing a clock rate
    /// higher than this will return an error. For a 44100 Hz sample rate the highest supported
    /// clock rate is 5.6448 MHz, well above the most popular PSG clock rates.
    ///
    /// By default the PSG is configured to emulate a Yamaha YM2149, but this can be changed
    /// afterwards by calling [`set_chip_type`](Self::set_chip_type).
    pub fn new(clock_rate: f64, sample_rate: u32) -> Result<Self, Error> {
        // First compute the step value to determine if it is within bounds
        let step = clock_rate / (sample_rate as f64 * 8.0 * DECIMATE_FACTOR as f64);

        if step >= 1.0 {
            return Err(Error::ClockRateTooHigh);
        }

        Ok(Self {
            channels: [Channel::new(), Channel::new(), Channel::new()],
            noise_generator: NoiseGenerator::new(),
            envelope_generator: EnvelopeGenerator::new(),

            log2lin_table: ChipType::YM.log2lin_table(),

            x: 0.0,
            step,

            left_interpolator: Interpolator::new(),
            right_interpolator: Interpolator::new(),

            left_decimator: Decimator::new(),
            right_decimator: Decimator::new(),
            decimator_index: 0,

            dc_filter: DCFilter::new()
        })
    }

    /// Set the PSG chip type to the specified type.
    ///
    /// This only affects the envelope generator resolution, which is higher for the Yamaha YM2149.
    pub fn set_chip_type(&mut self, chip_type: ChipType) {
        self.log2lin_table = chip_type.log2lin_table();
    }

    /// Render the next PSG clock tick.
    ///
    /// Returns a tuple containing the left channel as the first element and the right channel as
    /// the second.
    fn render_tick(&mut self) -> (f64, f64) {
        let noise = self.noise_generator.render();
        let envelope = self.envelope_generator.render();

        self.channels.iter_mut().fold((0.0, 0.0), |(left, right), channel| {
            let mut level = (channel.render() | channel.tone_off as u8) & (noise | channel.noise_off as u8);

            level *= if channel.envelope_on {
                envelope
            } else {
                channel.amplitude * 2 + 1
            };

            let amplitude = self.log2lin_table[level as usize];

            (left + amplitude * channel.pan_left, right + amplitude * channel.pan_right)
        })
    }

    /// Render the next frame.
    ///
    /// Returns a tuple containing the left channel as the first element and the right channel as
    /// the second.
    pub fn render(&mut self) -> (f64, f64) {
        let decimator_start = FIR_SIZE - self.decimator_index * DECIMATE_FACTOR;

        // modulo 23
        self.decimator_index = (self.decimator_index + 1) % (FIR_SIZE / DECIMATE_FACTOR - 1);

        // Fill decimator buffers in reverse
        // TODO: Since the filter is symmetrical, does this matter?
        for offset in (0..DECIMATE_FACTOR).rev() {
            self.x += self.step;

            if self.x >= 1.0 {
                self.x -= 1.0;

                let (left, right) = self.render_tick();

                self.left_interpolator.feed(left);
                self.right_interpolator.feed(right);
            }

            self.left_decimator.buffer[decimator_start + offset] = self.left_interpolator.interpolate(self.x);
            self.right_decimator.buffer[decimator_start + offset] = self.right_interpolator.interpolate(self.x);
        }

        self.dc_filter.render(
            self.left_decimator.render(decimator_start),
            self.right_decimator.render(decimator_start)
        )
    }

    /// Return a reference to the specified channel number's [`Channel`] struct.
    ///
    /// The channel number must be smaller than 3.
    pub fn channel(&self, index: u8) -> &Channel {
        &self.channels[index as usize]
    }

    /// Return a mutable reference to the specified channel number's [`Channel`] struct.
    ///
    /// The channel number must be smaller than 3.
    pub fn channel_mut(&mut self, index: u8) -> &mut Channel {
        &mut self.channels[index as usize]
    }

    /// Return a reference to the PSG's noise generator.
    pub fn noise_generator(&self) -> &NoiseGenerator {
        &self.noise_generator
    }

    /// Return a mutable reference to the PSG's noise generator.
    pub fn noise_generator_mut(&mut self) -> &mut NoiseGenerator {
        &mut self.noise_generator
    }

    /// Return a reference to the PSG's envelope generator.
    pub fn envelope_generator(&self) -> &EnvelopeGenerator {
        &self.envelope_generator
    }

    /// Return a mutable reference to the PSG's envelope generator.
    pub fn envelope_generator_mut(&mut self) -> &mut EnvelopeGenerator {
        &mut self.envelope_generator
    }

    /// Set a channel's tone period to a value between 1 and 4095 inclusive.
    ///
    /// Smaller values are set to 1, larger values are wrapped. The channel number must be smaller
    /// than 3.
    pub fn set_tone_period(&mut self, channel: u8, period: u16) {
        self.channels[channel as usize].set_period(period);
    }

    /// Set a channel's amplitude to a value between 0 and 15 inclusive.
    ///
    /// Larger values are wrapped. The channel number must be smaller than 3.
    pub fn set_amplitude(&mut self, channel: u8, amplitude: u8) {
        self.channels[channel as usize].set_amplitude(amplitude);
    }

    /// Set a channel's tone disable flag.
    ///
    /// The channel number must be smaller than 3.
    pub fn set_tone_disabled(&mut self, channel: u8, disabled: bool) {
        self.channels[channel as usize].set_tone_disabled(disabled);
    }

    /// Set a channel's noise disable flag.
    ///
    /// The channel number must be smaller than 3.
    pub fn set_noise_disabled(&mut self, channel: u8, disabled: bool) {
        self.channels[channel as usize].set_noise_disabled(disabled);
    }

    /// Set a channel's envelope enable flag.
    ///
    /// The channel number must be smaller than 3.
    pub fn set_envelope_enabled(&mut self, channel: u8, enabled: bool) {
        self.channels[channel as usize].set_envelope_enabled(enabled);
    }

    /// Set the noise generator's period to a value between 1 and 31 inclusive.
    ///
    /// Smaller values are set to 1, larger values are wrapped.
    pub fn set_noise_period(&mut self, period: u8) {
        self.noise_generator.set_period(period);
    }

    /// Set the PSG's mixer register value.
    ///
    /// The mixer value is an 8-bit number consisting of the following bits:
    ///
    /// Bit 0: Channel A tone enable (0 to enable, 1 to disable) \
    /// Bit 1: Channel B tone enable (0 to enable, 1 to disable) \
    /// Bit 2: Channel C tone enable (0 to enable, 1 to disable) \
    /// Bit 3: Channel A noise enable (0 to enable, 1 to disable) \
    /// Bit 4: Channel B noise enable (0 to enable, 1 to disable) \
    /// Bit 5: Channel C noise enable (0 to enable, 1 to disable) \
    /// Bit 6: GPIO In/out A toggle (ignored in this implementation) \
    /// Bit 7: GPIO In/out B toggle (ignored in this implementation)
    pub fn set_mixer(&mut self, mixer: u8) {
        self.channels[0].set_tone_disabled(mixer & 0x01 != 0);
        self.channels[1].set_tone_disabled(mixer & 0x02 != 0);
        self.channels[2].set_tone_disabled(mixer & 0x04 != 0);
        self.channels[0].set_noise_disabled(mixer & 0x08 != 0);
        self.channels[1].set_noise_disabled(mixer & 0x10 != 0);
        self.channels[2].set_noise_disabled(mixer & 0x20 != 0);

        // Note: the GPIO bits are ignored
    }

    /// Set the envelope generator period to a value between 1 and 65535 inclusive.
    ///
    /// Lower values are set to 1.
    pub fn set_envelope_period(&mut self, period: u16) {
        self.envelope_generator.set_period(period);
    }

    /// Set shape to a value between 0 and 15 inclusive.
    ///
    /// Higher values are wrapped.
    pub fn set_envelope_shape(&mut self, shape: u8) {
        self.envelope_generator.set_shape(shape);
    }

    /// Set a PSG register to the provided value.
    ///
    /// This function is particularly useful when writing emulators, as it provides a convenient
    /// API to the PSG's address/data ports that is easy to map from machine code.
    ///
    /// For an exact specification of the register numbers and their accepted values, please refer
    /// to the AY-3-8910 or YM2149 datasheets. Note that the AY-3-8910 datasheet uses octal numbers
    /// when referring to register numbers.
    ///
    /// Please note that the GPIO registers (14 and 15) are ignored in this implementation, and
    /// that writing to any register number higher than 15 will have no effect.
    pub fn set_register(&mut self, register: u8, value: u8) {
        // Note: the AY-3-8910 datasheet uses octal register numbers. The YM2149 datasheet uses
        // decimal numbers.
        match register {
            0 => self.channels[0].set_period_lsb(value),
            1 => self.channels[0].set_period_msb(value),
            2 => self.channels[1].set_period_lsb(value),
            3 => self.channels[1].set_period_msb(value),
            4 => self.channels[2].set_period_lsb(value),
            5 => self.channels[2].set_period_msb(value),
            6 => self.noise_generator.set_period(value),
            7 => self.set_mixer(value),
            8 => self.channels[0].set_amplitude_and_envelope_enabled(value),
            9 => self.channels[1].set_amplitude_and_envelope_enabled(value),
            10 => self.channels[2].set_amplitude_and_envelope_enabled(value),
            11 => self.envelope_generator.set_period_lsb(value),
            12 => self.envelope_generator.set_period_msb(value),
            13 => self.envelope_generator.set_shape(value),
            14 => (), // GPIO port A data store is ignored here
            15 => (), // GPIO port B data store is ignored here
            _ => ()
        }
    }
}
