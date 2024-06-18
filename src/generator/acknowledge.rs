use super::{AcknowledgedCounter, Counter, Generator};

use paste::paste;

use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize, Ordering,
};

use itertools::Itertools;
use parking_lot::RwLock;

macro_rules! acknowledged {
    ($( {$atype:ty, $type:ty, $name:ident}, )*) => {
        paste! {
            $(
                // FIXME(MrCroxx): Take care of the memory order.
                /// Generates a sequence of numeric value in an atomic manner.
                #[derive(Debug)]
                pub struct [<Acknowledged $name Counter>] {
                    counter: $atype,
                    windows: Vec<AtomicBool>,
                    limit: RwLock<$type>
                }

                impl [<Acknowledged $name Counter>] {
                    const WINDOW_SIZE: usize = 1 << 20;
                    const WINDOW_MASK: usize = Self::WINDOW_SIZE - 1;

                    /// Create a counter that starts at `start`.
                    pub fn new(start: $type) -> Self {
                        Self {
                            counter: $atype::new(start),
                            windows: (0..Self::WINDOW_SIZE).map(|_| AtomicBool::new(false)).collect_vec(),
                            limit: RwLock::new(start - 1),
                        }
                    }
                }

                impl Generator for [<Acknowledged $name Counter>] {
                    type Output = $type;

                    fn next(&self) -> Self::Output {
                        self.counter.fetch_add(1, Ordering::Relaxed)
                    }
                }

                impl Counter for [<Acknowledged $name Counter>] {
                    fn last(&self) -> Self::Output {
                        *self.limit.read()
                    }
                }

                impl AcknowledgedCounter for [<Acknowledged $name Counter>] {
                    fn acknowledge(&self, val: Self::Output) {
                        let slot = val as usize & Self::WINDOW_MASK;
                        if self.windows[slot].fetch_or(true, Ordering::SeqCst) {
                            panic!("Too many unacknowledged insertion keys.");
                        }

                        if let Some(mut limit) = self.limit.try_write() {
                            let stop = *limit as usize & Self::WINDOW_MASK;
                            let mut index = *limit + 1;
                            while index as usize & Self::WINDOW_MASK != stop {
                                let slot = index as usize & Self::WINDOW_MASK;
                                if !self.windows[slot].load(Ordering::SeqCst) {
                                    break;
                                }
                                self.windows[slot].store(false, Ordering::SeqCst);
                                index += 1;
                            }
                            *limit = index - 1;
                        }
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

for_all_numeric_types! { acknowledged }
