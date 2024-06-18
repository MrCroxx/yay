use super::{Generator, NumberGenerator};

use paste::paste;

use std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU64,
    AtomicU8, AtomicUsize, Ordering,
};

macro_rules! sequential {
    ($( {$atype:ty, $type:ty, $name:ident}, )*) => {
        paste! {
            $(
                /// Generates a sequence from start to end (included).
                #[derive(Debug)]
                pub struct [<Sequential $name Generator>] {
                    start: $type,
                    end: $type,
                    val: $atype,
                }

                impl [<Sequential $name Generator>] {
                    /// Create a new sequential generator that generates a sequence from start to end (included).
                    pub fn new(start: $type, end: $type) -> Self {
                        Self {
                            start,
                            end,
                            val: $atype::new(start),
                        }
                    }
                }

                impl Generator for [<Sequential $name Generator>] {
                    type Output = $type;

                    fn next(&self) -> Self::Output {
                        let val = self.val.fetch_add(1, Ordering::Relaxed);
                        self.start + (val % (self.end - self.start + 1))
                    }

                }

                impl NumberGenerator for [<Sequential $name Generator>] {
                    fn mean(&self) -> f64 {
                        (self.start as f64 + self.end as f64) / 2.0
                    }
                }
            )*
        }
    };
}

macro_rules! for_all_numeric_types {
    ($macro:ident) => {
        $macro! {
            {AtomicU8, u8, U8},
            {AtomicU16, u16, U16},
            {AtomicU32, u32, U32},
            {AtomicU64, u64, U64},
            {AtomicUsize, usize, Usize},
            {AtomicI8, i8, I8},
            {AtomicI16, i16, I16},
            {AtomicI32, i32, I32},
            {AtomicI64, i64, I64},
            {AtomicIsize, isize, Isize},
        }
    };
}

for_all_numeric_types! { sequential }
