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
