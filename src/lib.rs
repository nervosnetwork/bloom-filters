use std::hash::Hash;

mod buckets;
mod classic;
#[cfg(feature = "const_generics")]
mod const_generics;
mod counting;
mod hash;
mod stable;

pub use crate::classic::Filter as ClassicBloomFilter;
#[cfg(feature = "const_generics")]
pub use crate::const_generics::{
    buckets::{approximate_bucket_count, compute_word_num},
    classic::Filter as ConstClassicBloomFilter,
    stable::Filter as ConstStableBloomFilter,
};
pub use crate::counting::Filter as CountingBloomFilter;
pub use crate::hash::{BuildHashKernels, DefaultBuildHashKernels, DefaultBuildHasher, DefaultHashKernels, HashKernels};
pub use crate::stable::Filter as StableBloomFilter;

pub trait BloomFilter {
    fn insert<T: Hash>(&mut self, item: &T);
    fn contains<T: Hash>(&self, item: &T) -> bool;
    fn reset(&mut self);
}

pub trait RemovableBloomFilter {
    fn remove<T: Hash>(&mut self, item: &T);
}

pub trait UpdatableBloomFilter {
    /// Update filter internal buckets with `raw_data` via `BitOr` operation
    fn update(&mut self, raw_data: &[u8]);
}
