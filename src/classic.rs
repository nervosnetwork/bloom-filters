use crate::buckets::Buckets;
use crate::{BloomFilter, BuildHashKernels, HashKernels, UpdatableBloomFilter};
use std::hash::Hash;

pub struct Filter<BHK: BuildHashKernels> {
    buckets: Buckets,      // filter data
    hash_kernels: BHK::HK, // hash kernels
}

impl<BHK: BuildHashKernels> Filter<BHK> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new(items_count: usize, fp_rate: f64, build_hash_kernels: BHK) -> Self {
        let buckets = Buckets::with_fp_rate(items_count, fp_rate, 1);
        let hash_kernels = build_hash_kernels.with_fp_rate(fp_rate, buckets.len());
        Self { buckets, hash_kernels }
    }

    pub fn with_raw_data(raw_data: &[u8], k: usize, build_hash_kernels: BHK) -> Self {
        let buckets = Buckets::with_raw_data(raw_data.len() * 8, 1, raw_data);
        let hash_kernels = build_hash_kernels.with_k(k, buckets.len());
        Self { buckets, hash_kernels }
    }

    pub fn buckets(&self) -> &Buckets {
        &self.buckets
    }
}

impl<BHK: BuildHashKernels> BloomFilter for Filter<BHK> {
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

impl<BHK: BuildHashKernels> UpdatableBloomFilter for Filter<BHK> {
    fn update(&mut self, raw_data: &[u8]) {
        self.buckets.update(raw_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{DefaultBuildHashKernels, DefaultBuildHasher};
    use proptest::{collection::size_range, prelude::any_with, proptest, proptest_helper};
    use rand::random;
    use std::collections::hash_map::RandomState;

    fn _contains(items: &[usize]) {
        let mut filter = Filter::new(100, 0.03, DefaultBuildHashKernels::new(random(), RandomState::new()));
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

    fn _raw_data(items: &[usize]) {
        let data = vec![0; 8];
        let hash_seed = random();
        let mut filter = Filter::with_raw_data(&data, 2, DefaultBuildHashKernels::new(hash_seed, DefaultBuildHasher));
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        let data = filter.buckets().raw_data();
        let filter = Filter::with_raw_data(&data, 2, DefaultBuildHashKernels::new(hash_seed, DefaultBuildHasher));
        assert!(items.iter().all(|i| filter.contains(i)));
    }

    proptest! {
        #[test]
        fn raw_data(ref items in any_with::<Vec<usize>>(size_range(8).lift())) {
            _raw_data(items)
        }
    }

    fn _update(items1: &[usize], items2: &[usize]) {
        let data = vec![0; 8];
        let hash_seed = random();

        let mut filter1 = Filter::with_raw_data(&data, 2, DefaultBuildHashKernels::new(hash_seed, DefaultBuildHasher));
        items1.iter().for_each(|i| filter1.insert(i));

        let mut filter2 = Filter::with_raw_data(&data, 2, DefaultBuildHashKernels::new(hash_seed, DefaultBuildHasher));
        items2.iter().for_each(|i| filter2.insert(i));

        filter1.update(&filter2.buckets().raw_data());
        assert!(items1.iter().all(|i| filter1.contains(i)));
        assert!(items2.iter().all(|i| filter1.contains(i)));
    }

    proptest! {
        #[test]
        fn update(
            ref items1 in any_with::<Vec<usize>>(size_range(8).lift()),
            ref items2 in any_with::<Vec<usize>>(size_range(8).lift())
        ) {
            _update(items1, items2)
        }
    }
}
