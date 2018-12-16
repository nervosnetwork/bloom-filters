use std::hash::Hash;

mod buckets;
mod classic;
mod counting;
mod hash;
mod stable;

pub use crate::classic::Filter as ClassicBloomFilter;
pub use crate::counting::Filter as CountingBloomFilter;
pub use crate::hash::{DefaultHashKernals, HashKernals};
pub use crate::stable::Filter as StableBloomFilter;

pub trait BloomFilter {
    fn insert<T: Hash>(&mut self, item: &T);
    fn contains<T: Hash>(&self, item: &T) -> bool;
    fn reset(&mut self);
}

pub trait RemovableBloomFilter {
    fn remove<T: Hash>(&mut self, item: &T);
}
