/// The shape of an envelope segment.
enum EnvelopeShape {
    /// Slide down from 31 to 0 and progress to the next shape afterwards.
    SlideDown,

    /// Slide up from 0 to 31 and progress to the next shape afterwards.
    SlideUp,

    /// Hold the value at 31 and stay in this state indefinitely.
    HoldTop,

    /// Hold the value at 0 stay in this state indefinitely.
    HoldBottom
}

/// A table containing all possible PSG envelope shapes.
///
/// Each envelope is represented as two parts (states), and can either oscillate between the first
/// and second state, or transition from the first into the second state and stay there
/// indefinitely. The choice of behavior depends on the value of the second state.
const ENVELOPE_TABLE: [[EnvelopeShape; 2]; 16] = [
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideDown, EnvelopeShape::SlideDown],
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldBottom],
    [EnvelopeShape::SlideDown, EnvelopeShape::SlideUp],
    [EnvelopeShape::SlideDown, EnvelopeShape::HoldTop],
    [EnvelopeShape::SlideUp,   EnvelopeShape::SlideUp],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldTop],
    [EnvelopeShape::SlideUp,   EnvelopeShape::SlideDown],
    [EnvelopeShape::SlideUp,   EnvelopeShape::HoldBottom]
];

/// The PSG's envelope generator.
pub struct EnvelopeGenerator {
    position: u16,
    period: u16,
    shape: u8,
    segment: u8,
    value: u8
}

impl EnvelopeGenerator {
    /// Initialize a new envelope generator.
    pub(crate) fn new() -> Self {
        Self {
            position: 0,
            period: 1,
            shape: 0,
            segment: 0,
            value: 0
        }
    }

    /// Render the next clock tick for the envelope generator.
    ///
    /// Returns a byte containing the next envelope level.
    pub(crate) fn render(&mut self) -> u8 {
        self.position += 1;

        if self.position >= self.period {
            self.position = 0;

            match ENVELOPE_TABLE[self.shape as usize][self.segment as usize] {
                EnvelopeShape::SlideDown => {
                    if self.value == 0 {
                        self.segment ^= 1;
                        self.reset_segment();
                    } else {
                        self.value -= 1;
                    }
                }

                EnvelopeShape::SlideUp => {
                    if self.value >= 31 {
                        self.segment ^= 1;
                        self.reset_segment();
                    } else {
                        self.value += 1;
                    }
                }

                _ => ()
            }
        }

        self.value
    }

    /// Reset the envelope generator's value based on the current envelope shape.
    ///
    /// The value is set to 31 when the shape starts at a high value, and 0 otherwise.
    fn reset_segment(&mut self) {
        self.value = match ENVELOPE_TABLE[self.shape as usize][self.segment as usize] {
            EnvelopeShape::SlideDown | EnvelopeShape::HoldTop => 31,
            _ => 0
        };
    }

    /// The envelope generator's period.
    pub fn period(&self) -> u16 {
        self.period
    }

    /// Set the envelope generator's period to a value between 1 and 65535 inclusive.
    ///
    /// Lower values are set to 1.
    pub fn set_period(&mut self, period: u16) {
        self.period = period.max(1);
    }

    /// The most significant byte for the envelope generator's period.
    pub fn period_msb(&self) -> u8 {
        (self.period >> 8) as u8
    }

    /// Set the envelope generator's most significant byte of the period to a value between 0 and
    /// 255 inclusive.
    ///
    /// Setting this byte to zero when the least significant byte is also set to zero will result
    /// in the period being set to 1. It is therefore recommended to always set the most
    /// significant byte first.
    pub fn set_period_msb(&mut self, period: u8) {
        self.period = ((self.period & 0x00ff) | (((period as u16) & 0xff) << 8)).max(1);
    }

    /// The least significant byte for the envelope generator's period.
    pub fn period_lsb(&self) -> u8 {
        (self.period & 0xff) as u8
    }

    /// Set the envelope generator's least significant byte of the period to a value between 0 and
    /// 255 inclusive.
    ///
    /// Setting this byte to zero when the most significant byte is also set to zero will result in
    /// the period being set to 1. It is therefore recommended to always set the most significant
    /// byte first.
    pub fn set_period_lsb(&mut self, period: u8) {
        self.period = ((self.period & 0xff00) | (period as u16)).max(1);
    }

    /// The envelope generator's shape.
    pub fn shape(&self) -> u8 {
        self.shape
    }

    /// Set the envelope generator's shape to a value between 0 and 15 inclusive.
    ///
    /// Higher values are wrapped.
    ///
    /// For the exact specification of the envelope shapes, please refer to the AY-3-8912 or YM2149
    /// datasheets.
    pub fn set_shape(&mut self, shape: u8) {
        self.shape = shape & 0x0f;
        self.position = 0;
        self.segment = 0;
        self.reset_segment();
    }
}
