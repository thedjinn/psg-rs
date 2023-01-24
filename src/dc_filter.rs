/// A two channel DC-offset elimination filter.
///
/// Essentialy this is just a moving average filter where the average is subtracted from the input
/// signal.
pub struct DCFilter<const S: usize = 1024> {
    left_sum: f64,
    right_sum: f64,
    left_delay: [f64; S],
    right_delay: [f64; S],
    index: usize
}

impl<const S: usize> DCFilter<S> {
    /// Initialize a new DC-offset elimination filter.
    pub fn new() -> Self {
        Self {
            left_sum: 0.0,
            right_sum: 0.0,
            left_delay: [0.0; S],
            right_delay: [0.0; S],
            index: 0
        }
    }

    /// Render a new frame for the provided input samples.
    ///
    /// The result is a tuple containing the filtered left channel as the first element and the
    /// filtered right channel as the second element.
    pub fn render(&mut self, left: f64, right: f64) -> (f64, f64) {
          self.left_sum += -self.left_delay[self.index] + left;
          self.right_sum += -self.right_delay[self.index] + right;

          self.left_delay[self.index] = left;
          self.right_delay[self.index] = right;

          self.index = (self.index + 1) & (S - 1);

          (
              left - self.left_sum * (1.0 / S as f64),
              right - self.right_sum * (1.0 / S as f64)
          )
    }
}
