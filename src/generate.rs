use crate::ArrowStringArray;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone, PartialEq)]
pub enum RandomStringMode {
    Numeric,
    AlphaNumeric,
    AsciiPrintable,
}

pub fn generate_random_strings(
    num_string: usize,
    string_len: usize,
    prefix: &str,
    mode: RandomStringMode,
    seed: u64,
) -> ArrowStringArray {
    let mut rng = StdRng::seed_from_u64(seed);
    let alphabet = match mode {
        RandomStringMode::Numeric => (b'0'..=b'9').collect::<Vec<u8>>(),
        RandomStringMode::AlphaNumeric => (b'0'..=b'9').chain(b'A'..=b'Z').collect::<Vec<u8>>(),
        RandomStringMode::AsciiPrintable => (33..=126).chain(b'A'..=b'Z').collect::<Vec<u8>>(),
    };
    ArrowStringArray::from_iter((0..num_string).map(|_| {
        prefix
            .chars()
            .chain((0..string_len).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char))
            .collect::<String>()
    }))
}

pub fn generate_random_floats(
    num_floats: usize,
    min: u64,
    max: u64,
    denominator: u64,
    seed: u64,
) -> Vec<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..num_floats)
        .map(|_| rng.gen_range(min..max) as f64 / denominator as f64)
        .collect()
}
