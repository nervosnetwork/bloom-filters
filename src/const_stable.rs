//! Stable Bloom Filter Implementation with Const Generics
//! 
//! In some cases of using bloom filter, the memory size of bloom filter can be determined
//! in `compile time`. So it's an efficient way to implement bloom filter data structure with `const generics`,
//! which is stable in rust 1.51 version.
//!  
//! Compared to implementation using `Vec<T>`, there are some advantages:  
//! + The metadata is placed on the `stack` instead of `heap`, it will reduce some cost of `runtime`
//! + More elegant way to manage memory
//! 
//! However, there's also some disadvantages:
//! + Due to floating point arithmetic has not allowed in const fn yet, 
//! we should compute the num of bucket by hand
//! + Less functionality
//! 
//! Even so, it makes sence to implemet bloom filter with const generics.
//! 
//! example:
//! `cargo.toml`:  
//! bloom-filters = { git = "https://github.com/nervosnetwork/bloom-filters", features = ["const_generic"]}
//! rand = rand = "0.6"
//! 
//! ```Rust
//! use std::collections::hash_map::RandomState;
//! use rand::{random, thread_rng, Rng};
//! use rand::distributions::Standard;
//! use bloom_filters::{BloomFilter, ConstStableBloomFilter, DefaultBuildHashKernels, compute_word_num, filter};
//! fn main() {
//!     // item count: 10
//!     // bucket size: 3
//!     // fp rate: 0.03
//!     // bucket count = -10 * ln(0.03) / ln2 ^ 2 = 72.9844, we need to compute the bucket count by hand!
//!     let mut filter = filter!(
//!        73, 3, 0.03, DefaultBuildHashKernels::new(random(), RandomState::new())
//!     );
//!     let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
//!     items.iter().for_each(|i| filter.insert(i));
//!     let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
//!     let _ret: Vec<bool> = items.iter().map(|i| filter.contains(i)).collect();    
//! }
//! ```
//! 
use crate::const_buckets::ConstBuckets;
use crate::hash::compute_k_num;
use crate::{BloomFilter, BuildHashKernels, HashKernels};
use rand::random;
use std::hash::Hash;

#[derive(Clone)]
pub struct Filter<BHK: BuildHashKernels, const W: usize, const M: usize, const D: u8> {
    buckets: ConstBuckets<W, M, D>,      // filter data
    hash_kernels: BHK::HK, // hash kernels
    p: usize,              // number of buckets to decrement,
}

impl<BHK: BuildHashKernels, const W: usize, const M: usize, const D: u8> Filter<BHK, W, M, D> {
    /// Creates a new Stable Bloom Filter with m buckets and d
    /// bits allocated per bucket optimized for the target false-positive rate.
    pub fn new(fp_rate: f64, build_hash_kernels: BHK) -> Self {
        let mut k = compute_k_num(fp_rate);
        if k > M {
            k = M
        } else if k == 0 {
            k = 1
        }

        let buckets = ConstBuckets::new();
        let hash_kernels = build_hash_kernels.with_k(k, buckets.len());

        Self {
            buckets,
            hash_kernels,
            p: compute_p_num(M, k, D, fp_rate),
        }
    }

    pub fn buckets(&self) -> &ConstBuckets<W, M, D> {
        &self.buckets
    }

    fn decrement(&mut self) {
        let r: usize = random();
        (0..self.p).for_each(|i| {
            let bucket = (r + i) % self.buckets.len();
            self.buckets.increment(bucket, -1)
        })
    }
}

// returns the optimal number of buckets to decrement, p, per
// iteration for the provided parameters of an SBF.
fn compute_p_num(m: usize, k: usize, d: u8, fp_rate: f64) -> usize {
    let (m, k, d) = (m as f64, k as f64, f64::from(d));
    let max = 2f64.powf(d) - 1.0;
    let sub_denom = (1.0 - fp_rate.powf(1.0 / k)).powf(1.0 / max);
    let denom = (1.0 / sub_denom - 1.0) * (1.0 / k - 1.0 / m);
    let p = (1.0 / denom).ceil();
    if p <= 0.0 {
        1
    } else {
        p as usize
    }
}

impl<BHK: BuildHashKernels, const W: usize, const B: usize, const S: u8> BloomFilter for Filter<BHK, W, B, S> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.decrement();
        let max = self.buckets.max_value();
        self.hash_kernels.hash_iter(item).for_each(|i| self.buckets.set(i, max))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernels.hash_iter(item).all(|i| self.buckets.get(i) > 0)
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

#[macro_export]
macro_rules! filter {
    (
        $bucket_count:expr, $bucket_size:expr,
        $fp_rate:expr, $build_hash_kernels:expr
    ) => {
        ConstStableBloomFilter::<_, {compute_word_num($bucket_count, $bucket_size)}, $bucket_count, $bucket_size>::new($fp_rate, $build_hash_kernels)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::DefaultBuildHashKernels;
    use proptest::{collection::size_range, prelude::any_with, proptest};
    use rand::random;
    use std::collections::hash_map::RandomState;
    use crate::const_buckets::compute_word_num;
    fn _contains(items: &[usize]) {
        let mut filter = Filter::<_, {compute_word_num(730, 3)}, 730, 3>::new(0.03, DefaultBuildHashKernels::new(random(), RandomState::new()));
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        assert!(items.iter().all(|i| filter.contains(i)));
    }

    proptest! {
        #[test]
        fn contains(ref items in any_with::<Vec<usize>>(size_range(7).lift())) {
            _contains(items)
        }
    }
}
