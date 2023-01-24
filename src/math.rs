//! This module contains useful mathematical operations on frequencies, tone/envelope periods, and
//! MIDI pitch numbers.

/// Convert a MIDI pitch number into its corresponding frequency.
///
/// The pitch number is not required to be an integer.
pub fn midi_pitch_to_frequency(pitch: f64) -> f64 {
    (2.0_f64).powf((pitch - 69.0) / 12.0) * 440.0
}

/// Convert a frequency to its corresponding MIDI pitch number.
///
/// The resulting pitch number is not guaranteed to be an integer.
pub fn frequency_to_midi_pitch(frequency: f64) -> f64 {
    (frequency / 440.0).log2() * 12.0 + 69.0
}

/// Convert a MIDI pitch number into a suitable tone period value for the specified clock rate.
///
/// The pitch number is not required to be an integer.
pub fn midi_pitch_to_tone_period(pitch: f64, clock_rate: f64) -> u16 {
    frequency_to_tone_period(midi_pitch_to_frequency(pitch), clock_rate)
}

/// Convert a MIDI pitch number into a suitable envelope period value for the specified clock rate.
///
/// The pitch number is not required to be an integer.
pub fn midi_pitch_to_envelope_period(pitch: f64, clock_rate: f64) -> u16 {
    frequency_to_envelope_period(midi_pitch_to_frequency(pitch), clock_rate)
}

/// Convert a tone period value into its corresponding MIDI pitch number for the specified clock
/// rate.
///
/// The resulting pitch number is not guaranteed to be an integer.
pub fn tone_period_to_midi_pitch(period: u16, clock_rate: f64) -> f64 {
    frequency_to_midi_pitch(tone_period_to_frequency(period, clock_rate))
}

/// Convert a envelope period value into its corresponding MIDI pitch number for the specified
/// clock rate.
///
/// The resulting pitch number is not guaranteed to be an integer.
pub fn envelope_period_to_midi_pitch(period: u16, clock_rate: f64) -> f64 {
    frequency_to_midi_pitch(envelope_period_to_frequency(period, clock_rate))
}

/// Convert a frequency into its corresponding tone period value for the specified clock rate.
pub fn frequency_to_tone_period(frequency: f64, clock_rate: f64) -> u16 {
    (clock_rate / (16.0 * frequency)).round() as u16
}

/// Convert a frequency into its corresponding envelope period value for the specified clock rate.
pub fn frequency_to_envelope_period(frequency: f64, clock_rate: f64) -> u16 {
    (clock_rate / (256.0 * frequency)).round() as u16
}

/// Convert a tone period value into its corresponding frequency for the specified clock rate.
pub fn tone_period_to_frequency(period: u16, clock_rate: f64) -> f64 {
    clock_rate / (period as f64 * 16.0)
}

/// Convert a envelope period value into its corresponding frequency for the specified clock rate.
pub fn envelope_period_to_frequency(period: u16, clock_rate: f64) -> f64 {
    clock_rate / (period as f64 * 256.0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn midi_pitch_to_frequency() {
        assert_eq!(super::midi_pitch_to_frequency(81.0), 880.0);
        assert_eq!(super::midi_pitch_to_frequency(69.0), 440.0);
        assert_eq!(super::midi_pitch_to_frequency(57.0), 220.0);
    }

    #[test]
    fn frequency_to_midi_pitch() {
        assert_eq!(super::frequency_to_midi_pitch(880.0), 81.0);
        assert_eq!(super::frequency_to_midi_pitch(440.0), 69.0);
        assert_eq!(super::frequency_to_midi_pitch(220.0), 57.0);
    }

    #[test]
    fn tone_period_midi_conversion() {
        let period = super::midi_pitch_to_tone_period(57.0, 4400000.0);
        let pitch = super::tone_period_to_midi_pitch(period, 4400000.0);

        assert_eq!(pitch, 57.0);

        let frequency = super::tone_period_to_midi_pitch(100, 4400000.0);
        let period = super::midi_pitch_to_tone_period(frequency, 4400000.0);

        assert_eq!(period, 100);
    }

    #[test]
    fn envelope_period_midi_conversion() {
        let period = super::midi_pitch_to_envelope_period(21.0, 4400000.0);
        let pitch = super::envelope_period_to_midi_pitch(period, 4400000.0);

        assert_eq!(pitch, 21.0);

        let frequency = super::envelope_period_to_midi_pitch(100, 4400000.0);
        let period = super::midi_pitch_to_envelope_period(frequency, 4400000.0);

        assert_eq!(period, 100);
    }

    #[test]
    fn tone_period_conversion() {
        let period = super::frequency_to_tone_period(100.0, 1000000.0);
        let frequency = super::tone_period_to_frequency(period, 1000000.0);

        println!("{} {}", period, frequency);

        assert_eq!(frequency, 100.0);

        let frequency = super::tone_period_to_frequency(100, 1000000.0);
        let period = super::frequency_to_tone_period(frequency, 1000000.0);

        assert_eq!(period, 100);
    }

    #[test]
    fn envelop_period_conversion() {
        let period = super::frequency_to_envelope_period(1.25, 1000000.0);
        let frequency = super::envelope_period_to_frequency(period, 1000000.0);

        println!("{} {}", period, frequency);

        assert_eq!(frequency, 1.25);

        let frequency = super::envelope_period_to_frequency(100, 1000000.0);
        let period = super::frequency_to_envelope_period(frequency, 1000000.0);

        assert_eq!(period, 100);
    }
}
