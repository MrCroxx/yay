use super::{Generator, NumberGenerator};

use paste::paste;

use rand::{thread_rng, Rng};

macro_rules! uniform {
    ($( {$type:ty, $name:ident}, )*) => {
        paste! {
            $(
                /// An expression that generates a random value in the specified range.
                #[derive(Debug)]
                pub struct [<Uniform $name Generator>] {
                    lower_bound: $type,
                    upper_bound: $type,
                }

                impl [<Uniform $name Generator>] {
                    /// Creates a generator that will return numerics uniformly randomly from the interval
                    /// [lower_bound,upper_bound] inclusive (that is, lower_bound and upper_bound are possible values).
                    pub fn new(lower_bound: $type, upper_bound: $type) -> Self {
                        Self {
                            lower_bound,
                            upper_bound,
                        }
                    }
                }

                impl Generator for [<Uniform $name Generator>] {
                    type Output = $type;

                    fn next(&self) -> Self::Output {
                        thread_rng().gen_range(self.lower_bound..=self.upper_bound)
                    }

                }

                impl NumberGenerator for [<Uniform $name Generator>] {
                    fn mean(&self) -> f64 {
                        (self.lower_bound as f64 + self.upper_bound as f64) / 2.0
                    }
                }
            )*
        }
    };
}

macro_rules! for_all_numeric_types {
    ($macro:ident) => {
        $macro! {
            {u8, U8},
            {u16, U16},
            {u32, U32},
            {u64, U64},
            {usize, Usize},
            {i8, I8},
            {i16, I16},
            {i32, I32},
            {i64, I64},
            {isize, Isize},
        }
    };
}

for_all_numeric_types! { uniform }
