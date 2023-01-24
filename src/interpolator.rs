/// A 2nd order 4-point parabolic interpolator with cached coefficient, allowing the same input
/// value (and its history) to be used for multiple intermediate points.
///
/// More concrete details about the interpolation algorithm can be found here:
/// http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
pub struct Interpolator {
    y: [f64; 4],
    coefficients: [f64; 3]
}

impl Interpolator {
    /// Initialize a new interpolator.
    pub fn new() -> Self {
        Self {
            y: [0.0; 4],
            coefficients: [0.0; 3],
        }
    }

    /// Feed a new value into the interpolator.
    pub fn feed(&mut self, input: f64) {
        self.y[0] = self.y[1];
        self.y[1] = self.y[2];
        self.y[2] = self.y[3];
        self.y[3] = input;

        let y1 = self.y[2] - self.y[0];

        self.coefficients[0] = 0.5 * self.y[1] + 0.25 * (self.y[0] + self.y[2]);
        self.coefficients[1] = 0.5 * y1;
        self.coefficients[2] = 0.25 * (self.y[3] - self.y[1] - y1);
    }

    /// Perform a new interpolation for the intermediate value x (0..=1).
    pub fn interpolate(&self, x: f64) -> f64 {
        (self.coefficients[2] * x + self.coefficients[1]) * x + self.coefficients[0]
    }
}
