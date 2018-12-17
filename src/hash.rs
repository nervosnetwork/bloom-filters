use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, Hash, Hasher};

// Calculates the optimal number of hash functions to use for a Bloom
// filter based on the desired rate of false positives.
pub fn compute_k_num(fp_rate: f64) -> usize {
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    fp_rate.log2().abs().ceil() as usize
}

/// A trait for creating hash iterator of item.
pub trait HashKernals {
    type HI: Iterator<Item = usize>;

    fn hash_iter<T: Hash>(&self, item: &T) -> Self::HI;
}

/// A trait for creating instances of [`HashKernals`].
pub trait BuildHashKernals
where
    Self: Sized,
{
    type HK: HashKernals;

    fn with_fp_rate(self, fp_rate: f64, n: usize) -> Self::HK {
        self.with_k(compute_k_num(fp_rate), n)
    }

    fn with_k(self, k: usize, n: usize) -> Self::HK;
}

/// Used to create a DefaultHashKernals instance.
pub struct DefaultBuildHashKernals<BH> {
    hash_seed: usize,
    build_hasher: BH,
}

impl<BH: BuildHasher> DefaultBuildHashKernals<BH> {
    pub fn new(hash_seed: usize, build_hasher: BH) -> Self {
        Self { hash_seed, build_hasher }
    }
}

impl<BH: BuildHasher> BuildHashKernals for DefaultBuildHashKernals<BH> {
    type HK = DefaultHashKernals<BH>;

    fn with_k(self, k: usize, n: usize) -> Self::HK {
        Self::HK {
            k,
            n,
            hash_seed: self.hash_seed,
            build_hasher: self.build_hasher,
        }
    }
}

/// A default implementation of [Kirsch-Mitzenmacher-Optimization](https://www.eecs.harvard.edu/~michaelm/postscripts/tr-02-05.pdf) hash function
pub struct DefaultHashKernals<BH> {
    k: usize,         // numbers of hash iterating
    n: usize,         // filter size
    hash_seed: usize, // seed offset for anonymity and privacy purpose
    build_hasher: BH,
}

impl<BH: BuildHasher> HashKernals for DefaultHashKernals<BH> {
    type HI = DefaultHashIter;

    fn hash_iter<T: Hash>(&self, item: &T) -> Self::HI {
        let hasher = &mut self.build_hasher.build_hasher();
        item.hash(hasher);
        let result = hasher.finish();

        DefaultHashIter::new(result, self.k, self.n, self.hash_seed)
    }
}

pub struct DefaultHashIter {
    h1: usize,
    h2: usize,
    k: usize,
    n: usize,
    hash_seed: usize,
    counter: usize,
}

impl DefaultHashIter {
    fn new(hash: u64, k: usize, n: usize, hash_seed: usize) -> Self {
        Self {
            h1: (hash as u32) as usize,
            h2: (hash >> 32) as usize,
            k,
            n,
            hash_seed,
            counter: 0,
        }
    }
}

impl Iterator for DefaultHashIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.k == self.counter {
            return None;
        }
        let r = self
            .hash_seed
            .wrapping_add(self.h1)
            .wrapping_add(self.h2.wrapping_mul(self.counter))
            % self.n;
        self.counter += 1;
        Some(r)
    }
}

pub struct DefaultBuildHasher;

impl BuildHasher for DefaultBuildHasher {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}
