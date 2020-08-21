use crate::buckets::Buckets;
use crate::{BloomFilter, BuildHashKernels, HashKernels, RemovableBloomFilter};
use std::hash::Hash;

pub struct Filter<BHK: BuildHashKernels> {
    buckets: Buckets,      // filter data
    hash_kernels: BHK::HK, // hash kernels
}

impl<BHK: BuildHashKernels> Filter<BHK> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// bucket_size is the specified number of bits
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new(items_count: usize, bucket_size: u8, fp_rate: f64, build_hash_kernels: BHK) -> Self {
        let buckets = Buckets::with_fp_rate(items_count, fp_rate, bucket_size);
        let hash_kernels = build_hash_kernels.with_fp_rate(fp_rate, buckets.len());
        Self { buckets, hash_kernels }
    }
}

impl<BHK: BuildHashKernels> BloomFilter for Filter<BHK> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.hash_kernels.hash_iter(item).for_each(|i| self.buckets.increment(i, 1))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernels.hash_iter(item).all(|i| self.buckets.get(i) > 0)
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

impl<BHK: BuildHashKernels> RemovableBloomFilter for Filter<BHK> {
    fn remove<T: Hash>(&mut self, item: &T) {
        self.hash_kernels.hash_iter(item).for_each(|i| self.buckets.increment(i, -1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::DefaultBuildHashKernels;
    use proptest::{collection::size_range, prelude::any, prelude::any_with, proptest};
    use rand::random;
    use std::collections::hash_map::RandomState;

    fn _contains(items: &[usize]) {
        let mut filter = Filter::new(100, 4, 0.03, DefaultBuildHashKernels::new(random(), RandomState::new()));
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

    fn _remove(item: usize) {
        let mut filter = Filter::new(100, 4, 0.03, DefaultBuildHashKernels::new(random(), RandomState::new()));
        filter.insert(&item);
        filter.remove(&item);
        assert!(!filter.contains(&item));
    }

    proptest! {
        #[test]
        fn remove(items in any::<usize>()) {
            _remove(items)
        }
    }
}
