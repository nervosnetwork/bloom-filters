use crate::const_generics::buckets::ConstBuckets;
use crate::{BloomFilter, BuildHashKernels, HashKernels};
use std::hash::Hash;

#[derive(Clone)]
pub struct Filter<BHK: BuildHashKernels, const W: usize> {
    buckets: ConstBuckets<W>, // filter data
    hash_kernels: BHK::HK,    // hash kernels
}

impl<BHK: BuildHashKernels, const W: usize> Filter<BHK, W> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    #[allow(unused)]
    pub fn new(items_count: usize, fp_rate: f64, build_hash_kernels: BHK) -> Self {
        let buckets = ConstBuckets::with_fp_rate(items_count, fp_rate, 1);
        let hash_kernels = build_hash_kernels.with_fp_rate(fp_rate, buckets.len());
        Self { buckets, hash_kernels }
    }

    // pub fn with_raw_data(raw_data: &[u8], k: usize, build_hash_kernels: BHK) -> Self {
    //     let buckets = ConstBuckets::with_raw_data(raw_data.len() * 8, 1, raw_data);
    //     let hash_kernels = build_hash_kernels.with_k(k, buckets.len());
    //     Self { buckets, hash_kernels }
    // }

    #[allow(unused)]
    pub fn buckets(&self) -> &ConstBuckets<W> {
        &self.buckets
    }
}

impl<BHK: BuildHashKernels, const W: usize> BloomFilter for Filter<BHK, W> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.hash_kernels.hash_iter(item).for_each(|i| self.buckets.set(i, 1))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernels.hash_iter(item).all(|i| self.buckets.get(i) == 1)
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

// Calculates the buckets count approximately(bigger than how many system needs)
#[macro_export]
macro_rules! classicfilter {
    (
        $items_count:expr,
        $fp_rate:expr,
        $build_hash_kernels:expr
    ) => {
        ConstClassicBloomFilter::<_, { compute_word_num(approximate_bucket_count($items_count), 1) }>::new(
            $items_count,
            $fp_rate,
            $build_hash_kernels,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::const_generics::buckets::{approximate_bucket_count, compute_word_num};
    use crate::hash::DefaultBuildHashKernels;
    use proptest::{collection::size_range, prelude::any_with, proptest};
    use rand::random;
    use std::collections::hash_map::RandomState;

    fn _contains(items: &[usize]) {
        let mut filter = Filter::<_, { compute_word_num(approximate_bucket_count(100), 1) }>::new(
            100,
            0.03,
            DefaultBuildHashKernels::new(random(), RandomState::new()),
        );
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        assert!(items.iter().all(|i| filter.contains(i)));
    }

    proptest! {
        #[test]
        fn contains(ref items in any_with::<Vec<usize>>(size_range(16).lift())) {
            _contains(items)
        }
    }
}
