extern crate rand;

use std::f64::consts::LN_2;
use std::hash::{Hash, Hasher};

mod buckets;
mod classic;
mod counting;
mod stable;

pub use classic::Filter as ClassicBloomFilter;
pub use counting::Filter as CountingBloomFilter;
pub use stable::Filter as StableBloomFilter;

pub trait BloomFilter {
    fn insert<T: Hash>(&mut self, item: &T);
    fn contains<T: Hash>(&self, item: &T) -> bool;
    fn reset(&mut self);
}

pub trait RemovableBloomFilter {
    fn remove<T: Hash>(&mut self, item: &T);
}

const LN_2_2: f64 = LN_2 * LN_2;

fn hash_kernals<T: Hash, H: Hasher>(item: &T, hasher: &mut H) -> (u32, u32) {
    item.hash(hasher);
    let result = hasher.finish();
    (result as u32, (result >> 32) as u32)
}

// Calculates the optimal Bloom filter size, m, based on the number of
// items and the desired rate of false positives.
fn compute_m_num(items_count: usize, fp_rate: f64) -> usize {
    assert!(items_count > 0);
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    ((items_count as f64) * fp_rate.ln().abs() / LN_2_2).ceil() as usize
}

// Calculates the optimal number of hash functions to use for a Bloom
// filter based on the desired rate of false positives.
fn compute_k_num(fp_rate: f64) -> usize {
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    fp_rate.log2().abs().ceil() as usize
}
