use crate::const_generics::buckets::ConstBuckets;
use crate::hash::compute_k_num;
use crate::{BloomFilter, BuildHashKernels, HashKernels};
use rand::random;
use std::hash::Hash;

#[derive(Clone)]
pub struct Filter<BHK: BuildHashKernels, const W: usize> {
    buckets: ConstBuckets<W>, // filter data
    hash_kernels: BHK::HK,    // hash kernels
    p: usize,                 // number of buckets to decrement,
}

impl<BHK: BuildHashKernels, const W: usize> Filter<BHK, W> {
    /// Creates a new Stable Bloom Filter with m buckets and d
    /// bits allocated per bucket optimized for the target false-positive rate.
    #[allow(unused)]
    pub fn new(bucket_count: usize, bucket_size: u8, fp_rate: f64, build_hash_kernels: BHK) -> Self {
        let mut k = compute_k_num(fp_rate);
        if k > bucket_count {
            k = bucket_count
        } else if k == 0 {
            k = 1
        }

        let buckets = ConstBuckets::new(bucket_count, bucket_size);
        let hash_kernels = build_hash_kernels.with_k(k, buckets.len());

        Self {
            buckets,
            hash_kernels,
            p: compute_p_num(bucket_count, k, bucket_size, fp_rate),
        }
    }

    #[allow(unused)]
    pub fn buckets(&self) -> &ConstBuckets<W> {
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

impl<BHK: BuildHashKernels, const W: usize> BloomFilter for Filter<BHK, W> {
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
macro_rules! stablefilter {
    (
        $bucket_count:expr, $bucket_size:expr,
        $fp_rate:expr, $build_hash_kernels:expr
    ) => {
        ConstStableBloomFilter::<_, { compute_word_num($bucket_count, $bucket_size) }>::new(
            $bucket_count,
            $bucket_size,
            $fp_rate,
            $build_hash_kernels,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::const_generics::buckets::compute_word_num;
    use crate::hash::DefaultBuildHashKernels;
    use proptest::{collection::size_range, prelude::any_with, proptest};
    use rand::random;
    use std::collections::hash_map::RandomState;
    fn _contains(items: &[usize]) {
        let mut filter = Filter::<_, { compute_word_num(730, 3) }>::new(
            730,
            3,
            0.03,
            DefaultBuildHashKernels::new(random(), RandomState::new()),
        );
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
