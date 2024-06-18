//  Copyright 2024 MrCroxx
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

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
