use super::{Generator, NumberGenerator};

use paste::paste;

macro_rules! constant {
    ($( {$type:ty, $name:ident}, )*) => {
        paste! {
            $(
                /// A tritival numeric generator that always returns the same value.
                #[derive(Debug)]
                pub struct [<Constant $name Generator>] {
                    val: $type,
                }

                impl [<Constant $name Generator>] {
                    /// Creates a tritival numeric generator that always returns the same value.
                    pub fn new(val: $type) -> Self {
                        Self {
                            val,
                        }
                    }
                }

                impl Generator for [<Constant $name Generator>] {
                    type Output = $type;

                    fn next(&self) -> Self::Output {
                        self.val
                    }
                }

                impl NumberGenerator for [<Constant $name Generator>] {
                    fn mean(&self) -> f64 {
                        self.val as f64
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

for_all_numeric_types! { constant }
