/// One of the YM-3-8910/YM2149's tone generator channels.
///
/// A channel represents a single square wave oscillator with configurable period, amplitude, and
/// panning.
///
/// The channel's signal is generated as the sum of the square oscillator and the chip's noise
/// generator, which can both be turned off independently. This signal is then multiplied with the
/// channel amplitude, which can either be a fixed value, or the chip's envelope generator output.
pub struct Channel {
    // Oscillator
    period: u16,
    position: u16,
    value: u8,

    // Flags
    pub(crate) tone_off: bool,
    pub(crate) noise_off: bool,
    pub(crate) envelope_on: bool,

    // Amplitude
    pub(crate) amplitude: u8,

    // Left/right panning
    pub(crate) pan_left: f64,
    pub(crate) pan_right: f64
}

impl Channel {
    /// Initialize a new channel.
    pub(crate) fn new() -> Self {
        Self {
            period: 1,
            position: 0,
            value: 0,

            tone_off: true,
            noise_off: true,
            envelope_on: false,

            amplitude: 0,

            pan_left: 0.5,
            pan_right: 0.5
        }
    }

    /// Produce a new sample for the channel's square wave oscillator.
    pub(crate) fn render(&mut self) -> u8 {
        self.position += 1;

        if self.position >= self.period {
            self.position = 0;
            self.value ^= 1;
        }

        self.value
    }

    /// The channel's tone period.
    ///
    /// This will return a value between 1 and 4095 inclusive.
    pub fn period(&self) -> u16 {
        self.period
    }

    /// Set the channel's tone period to a value between 1 and 4095 inclusive.
    ///
    /// Lower values are set to 1, higher values are wrapped.
    pub fn set_period(&mut self, period: u16) {
        self.period = (period & 0x0fff).max(1);
    }

    /// The most significant byte for the channel's tone period.
    ///
    /// This will return a value between 0 and 15 inclusive.
    pub fn period_msb(&self) -> u8 {
        (self.period >> 8) as u8
    }

    /// Set the most significant byte of the channel's tone period to a value between 0 and 15
    /// inclusive.
    ///
    /// Setting this byte to zero when the least significant byte is also set to zero will result
    /// in the period being set to 1. It is therefore recommended to always set the most
    /// significant byte first.
    ///
    /// Values higher than 15 will be wrapped.
    pub fn set_period_msb(&mut self, period: u8) {
        self.period = ((self.period & 0x00ff) | (((period as u16) & 0x0f) << 8)).max(1);
    }

    /// The least significant byte for the channel's tone period.
    pub fn period_lsb(&self) -> u8 {
        (self.period & 0xff) as u8
    }

    /// Set the least significant byte of the channel's tone period to a value between 0 and 255
    /// inclusive.
    ///
    /// Setting this byte to zero when the most significant byte is also set to zero will result in
    /// the period being set to 1. It is therefore recommended to always set the most significant
    /// byte first.
    pub fn set_period_lsb(&mut self, period: u8) {
        self.period = ((self.period & 0x0f00) | (period as u16)).max(1);
    }

    /// The channel's amplitude.
    ///
    /// This will return a value between 0 and 15 inclusive.
    pub fn amplitude(&self) -> u8 {
        self.amplitude
    }

    /// Set the channel's amplitude to a value between 0 and 15 inclusive.
    ///
    /// Higher values are wrapped.
    pub fn set_amplitude(&mut self, amplitude: u8) {
        self.amplitude = amplitude & 0x0f;
    }

    /// The channel's envelope enabled flag.
    pub fn envelope_enabled(&self) -> bool {
        self.envelope_on
    }

    /// Set the channel's envelope enabled flag.
    pub fn set_envelope_enabled(&mut self, enabled: bool) {
        self.envelope_on = enabled;
    }

    /// The channel's amplitude register.
    ///
    /// This is effectively a combination of the the amplitude and the envelope enabled flag using
    /// a single byte. In this byte the bits 0 through 4 are the amplitude and bit 5 is the
    /// envelope enabled flag.
    pub fn amplitude_and_envelope_enabled(&self) -> u8 {
        ((self.envelope_on as u8) << 4) | self.amplitude
    }

    /// Set the channel's amplitude to a value between 0 and 15 inclusive, taken from bits 0
    /// through 3 from the input value, and set the envelope enabled flag to the value of bit 4
    /// from the input value.
    ///
    /// This is equivalent to writing to the channel's amplitude register on a real PSG.
    pub fn set_amplitude_and_envelope_enabled(&mut self, value: u8) {
        self.amplitude = value & 0x0f;
        self.envelope_on = value & 0x10 != 0;
    }

    /// The channel's tone disabled flag.
    pub fn tone_disabled(&self) -> bool {
        self.tone_off
    }

    /// Set the channel's tone disabled flag.
    pub fn set_tone_disabled(&mut self, disabled: bool) {
        self.tone_off = disabled;
    }

    /// The channel's noise disabled flag.
    pub fn noise_disabled(&self) -> bool {
        self.noise_off
    }

    /// Set the channel's noise disabled flag.
    pub fn set_noise_disabled(&mut self, disabled: bool) {
        self.noise_off = disabled;
    }

    /// The channel's panning, represented as a scaling factor that is applied to the left channel
    /// (first value) and the right channel (second value).
    pub fn panning(&self) -> (f64, f64) {
        (self.pan_left, self.pan_right)
    }

    /// Set the channel's panning to a value between 0.0 (full left) and 1.0 (full right)
    /// inclusive.
    ///
    /// The `equal_power` argument can be set to true to interpret specified balance value as the
    /// ratio between each channel's power instead of amplitude. This effectively takes the square
    /// root of the balance and the square root of one minus the balance and applies these values
    /// as the panning factors.
    pub fn set_panning(&mut self, balance: f64, equal_power: bool) {
        self.pan_left = 1.0 - balance;
        self.pan_right = balance;

        if equal_power {
            self.pan_left = self.pan_left.sqrt();
            self.pan_right = self.pan_right.sqrt();
        }
    }
}
