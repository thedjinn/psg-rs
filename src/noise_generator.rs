/// The PSG's noise generator consists of a 17-bit linear feedback shift register with taps at bits
/// 13 and 16.
///
/// The register is updated once every N samples (the period).
pub struct NoiseGenerator {
    period: u8,
    counter: u8,
    value: u32
}

impl NoiseGenerator {
    /// Initialize a new noise generator.
    pub(crate) fn new() -> Self {
        Self {
            period: 1,
            counter: 0,

            // This initialization value is needed so that the generator's Galois LFSR will produce
            // identical results to Ayumi's Fibonacci LFSR, which starts at 1. Galois LFSRs are
            // faster to compute but introduce a delay, so compensation is needed.
            value: 0x4001
        }
    }

    /// Render the next tick for the noise generator.
    ///
    /// This returns a byte containing the next noise value. The actual noise value is a 1-bit
    /// number.
    pub(crate) fn render(&mut self) -> u8 {
        self.counter += 1;

        if self.counter >= (self.period << 1) {
            self.counter = 0;

            // Compute the next value of the LFSR in Galois form
            let lsb = self.value & 1;
            self.value = ((self.value >> 1) as i32 ^ ((-(lsb as i32)) & 0x12000)) as u32;
        }

        (self.value & 1) as u8
    }

    /// The noise generator's period.
    ///
    /// This returns a value between 1 and 31 inclusive.
    pub fn period(&self) -> u8 {
        self.period
    }

    /// Set the noise generator's period to a value between 1 and 31 inclusive.
    ///
    /// Lower values are set to 1, higher values are wrapped.
    pub fn set_period(&mut self, period: u8) {
        self.period = (period & 0x1f).max(1);
    }
}
