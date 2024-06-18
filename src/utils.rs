use std::io::{Cursor, Read, Write};

use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

const FNV_OFFSET_BASIS_64: u64 = 0xCBF29CE484222325;
const FNV_PRIME_64: u64 = 1099511628211;

/// http://en.wikipedia.org/wiki/Fowler_Noll_Vo_hash
///
/// ```plain
/// algorithm fnv-1 is
///     hash := FNV_offset_basis
///     
///     for each byte_of_data to be hashed do
///         hash := hash × FNV_prime
///         hash := hash XOR byte_of_data
///     
///     return hash
/// ```
pub fn fnvhash64(mut val: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS_64;

    for _ in 0..8 {
        let byte = val as u8;
        val >>= 8;

        hash = hash.wrapping_mul(FNV_PRIME_64);
        hash ^= byte as u64;
    }

    hash
}

/// Random lazy buf.
#[derive(Debug, Clone)]
pub struct RandomBytes {
    remaining: usize,
}

impl RandomBytes {
    /// Create a new random lazy buf with the given size.
    pub fn new(size: usize) -> Self {
        Self { remaining: size }
    }
}

impl Read for RandomBytes {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(buf.len(), self.remaining);
        self.remaining -= len;

        let s = Alphanumeric.sample_string(&mut thread_rng(), len);
        let written = buf.write(s.as_bytes())?;
        assert_eq!(len, written);
        Ok(len)
    }
}

/// Record value type.
#[derive(Debug, Clone)]
pub enum Value {
    /// Deterministic value type, which is used to check data inategrity.
    Deterministic(Cursor<String>),
    /// A random value type with minimum overhead.
    Random(RandomBytes),
}

impl Read for Value {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Value::Deterministic(c) => c.read(buf),
            Value::Random(r) => r.read(buf),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Deterministic(Cursor::new(value))
    }
}

impl From<RandomBytes> for Value {
    fn from(value: RandomBytes) -> Self {
        Self::Random(value)
    }
}
