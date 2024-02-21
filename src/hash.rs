use std::ops::BitXor;

#[inline(always)]
pub fn hash_bits_original(state: u64, value: u64) -> u64 {
    // Based on FxHasher (https://github.com/rust-lang/rustc-hash/blob/master/src/lib.rs)
    // with added swap_bytes
    state
        .rotate_left(5)
        .bitxor(value)
        .wrapping_mul(0x517cc1b727220a95)
}

#[inline(always)]
pub fn hash_bits_variant(state: u64, value: u64) -> u64 {
    // Based on FxHasher (https://github.com/rust-lang/rustc-hash/blob/master/src/lib.rs)
    // with added swap_bytes
    state
        .rotate_left(31)
        .swap_bytes()
        .bitxor(value)
        .wrapping_mul(0x517cc1b727220a95)
}

pub fn hash_key_generic<F: Fn(u64, u64) -> u64>(key: &[u8], merge_state: F) -> u64 {
    let chunks = key.chunks_exact(8);
    let mut remainder = chunks.remainder();

    let mut state = 0;

    for chunk in chunks {
        let mut buf = [0_u8; 8];
        buf.copy_from_slice(chunk);
        state = merge_state(state, u64::from_le_bytes(buf));
    }
    if !remainder.is_empty() {
        let mut remainder_bits = 0_u64;
        let mut shift = 0;
        if remainder.len() >= 4 {
            remainder_bits = u32::from_le_bytes(remainder[..4].try_into().unwrap()) as u64;
            remainder = &remainder[4..];
            shift += 32;
        }
        if remainder.len() >= 2 {
            remainder_bits |=
                (u16::from_le_bytes(remainder[..2].try_into().unwrap()) as u64) << shift;
            remainder = &remainder[2..];
            shift += 16;
        }
        if !remainder.is_empty() {
            remainder_bits |= (remainder[0] as u64) << shift;
        }
        state = merge_state(state, remainder_bits);
    }
    state
}

pub fn hash_key(key: &[u8]) -> u64 {
    hash_key_generic(key, hash_bits_variant)
}

#[inline(always)]
pub fn fold_bits(hash: u64) -> u64 {
    hash ^ (hash >> 32) ^ (hash >> 48)
}

#[cfg(test)]
mod tests {
    use crate::{generate_random_strings, hash_key, RandomStringMode};
    use hashbrown::HashSet;

    #[test]
    fn test_bit_histogram() {
        let mut histogram = [0_usize; 64];
        let strings = generate_random_strings(10_000, 8, "", RandomStringMode::Numeric, 1234);

        strings.iter().for_each(|s| {
            let h = hash_key(s.as_bytes());
            (0..64).for_each(|i| histogram[i] += (h & (1 << i) != 0) as usize);
        });

        histogram.iter().enumerate().for_each(|(i, count)| {
            println!("{i:02}: {count}");
        })
    }

    #[test]
    fn test_distinct_hashes() {
        let strings =
            generate_random_strings(100_000, 8, "12345678", RandomStringMode::Numeric, 1234);
        let hashes = strings
            .iter()
            .map(|s| hash_key(s.as_bytes()))
            .collect::<Vec<u64>>();
        let mut set = HashSet::new();

        for bits in 1..=20 {
            set.clear();
            hashes.iter().for_each(|h| {
                set.insert(h & (u64::MAX >> (64 - bits)));
            });
            println!("Lowest {bits:2} bits -> {:3} distinct hashes", set.len());
        }
        println!();

        for bits in 1..=10 {
            set.clear();
            hashes.iter().for_each(|h| {
                set.insert(h & (u64::MAX << (64 - bits)));
            });
            println!("Highest {bits:2} bits -> {:3} distinct hashes", set.len());
        }
    }
}
