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

use rand::{thread_rng, Rng};

use super::Generator;

/// Choice of the generated value of [`DiscreteGenerator`].
pub struct Choice<T> {
    /// Value to generate.
    pub val: T,
    /// Possibility weight of the choice.
    pub weight: f64,
}

/// Generates a distribution by choosing from a discrete set of values.
pub struct DiscreteGenerator<T> {
    choices: Vec<Choice<T>>,
    sum: f64,
}

impl<T> DiscreteGenerator<T> {
    /// Create a generator that generates a distribution by choosing from a discrete set of values.
    pub fn new(choices: Vec<Choice<T>>) -> Self {
        let sum = choices.iter().map(|choice| choice.weight).sum();
        Self { choices, sum }
    }
}

impl<T> Generator for DiscreteGenerator<T>
where
    T: Clone,
{
    type Output = T;

    fn next(&self) -> Self::Output {
        let target = thread_rng().gen_range(0.0..self.sum);
        let mut acc = 0.0;
        for choice in self.choices.iter() {
            acc += choice.weight;
            if target < acc {
                return choice.val.clone();
            }
        }
        unreachable!()
    }
}
