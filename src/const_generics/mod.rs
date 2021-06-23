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
//! bloom-filters = { git = "https://github.com/nervosnetwork/bloom-filters", features = ["const_generics"]}
//! rand = "0.6"
//!
//! ```Rust
//! use std::collections::hash_map::RandomState;
//! use rand::{random, thread_rng, Rng};
//! use rand::distributions::Standard;
//! use bloom_filters::{BloomFilter, ConstStableBloomFilter, DefaultBuildHashKernels, compute_word_num, stablefilter};
//! fn main() {
//!     // item count: 10
//!     // bucket size: 3
//!     // fp rate: 0.03
//!     // bucket count = -10 * ln(0.03) / ln2 ^ 2 = 72.9844, we need to compute the bucket count by hand!
//!     let mut filter = stablefilter!(
//!        73, 3, 0.03, DefaultBuildHashKernels::new(random(), RandomState::new())
//!     );
//!     let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
//!     items.iter().for_each(|i| filter.insert(i));
//!     let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
//!     let _ret: Vec<bool> = items.iter().map(|i| filter.contains(i)).collect();    
//! }
//! ```
//!
pub mod buckets;
pub mod stable;
