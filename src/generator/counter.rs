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

use super::{Counter, Generator};

use paste::paste;

use std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU64,
    AtomicU8, AtomicUsize, Ordering,
};

macro_rules! counter {
    ($( {$atype:ty, $type:ty, $name:ident}, )*) => {
        paste! {
            $(
                // FIXME(MrCroxx): Take care of the memory order.
                /// Generates a sequence of numeric value in an atomic manner.
                #[derive(Debug)]
                pub struct [<$name Counter>] {
                    counter: $atype,
                }

                impl [<$name Counter>] {
                    /// Create a counter that starts at `start`.
                    pub fn new(start: $type) -> Self {
                        Self {
                            counter: $atype::new(start),
                        }
                    }
                }

                impl Generator for [<$name Counter>] {
                    type Output = $type;

                    fn next(&self) -> Self::Output {
                        self.counter.fetch_add(1, Ordering::Relaxed)
                    }
                }

                impl Counter for [<$name Counter>] {
                    fn last(&self) -> Self::Output {
                        self.counter.load(Ordering::Relaxed) - 1
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

for_all_numeric_types! { counter }
