//! This example renders a one second 440 Hz tone with a sampling rate of 44100 Hz. This snippet of
//! audio is then written to a file called `output.raw` using a two channel little-endian f32
//! sample format.

use psg::PSG;
use psg::math;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize a new PSG with a clock rate of an MSX machine and a sampling rate of 44100 Hz.
    let clock_rate = 1789772.5;
    let mut psg = PSG::new(clock_rate, 44100).expect("Could not initialize PSG");

    // Set some registers.
    let channel = psg.channel_mut(0);
    channel.set_period(math::frequency_to_tone_period(440.0, clock_rate));
    channel.set_amplitude(15);
    channel.set_tone_disabled(false);

    // Open a file for writing
    let mut file = BufWriter::new(File::create("tone.raw")?);

    // Write a second of audio to the file
    for _ in 0..44100 {
        // Render the sample
        let (left, right) = psg.render();

        // Write samples as f32 in little-endian byte order
        file.write(&(left as f32).to_le_bytes())?;
        file.write(&(right as f32).to_le_bytes())?;
    }

    Ok(())
}
