/// A [`Generator`] generates value following some distribution.
pub trait Generator {
    /// Output type of the generator.
    type Output;

    /// Generate the next value.
    fn next(&self) -> Self::Output;
}

/// A [`NumberGenerator`] generates numeric values.
pub trait NumberGenerator: Generator {
    /// Return the expected value (mean) of the values this generator will generate.
    fn mean(&self) -> f64;
}

/// A [`Counter`] generates a sequence of integers.
pub trait Counter: Generator {
    /// Get the last generated value.
    ///
    /// # Panics
    ///
    /// `next()` must be called before calling `last()`.
    fn last(&self) -> Self::Output;
}

/// A [`AcknowledgedCounter`] only updates the last generated value with `acknowledge()` calls.
pub trait AcknowledgedCounter: Counter {
    /// Update the last generated value.
    fn acknowledge(&self, val: Self::Output);
}

/// Constant value generator.
pub mod constant;
/// Uniform value generator.
pub mod uniform;

/// Discrete value generator.
pub mod discrete;

/// Acknowledged atomic counters.
pub mod acknowledge;
/// Atomic counters.
pub mod counter;
/// Sequential generator.
pub mod sequential;
