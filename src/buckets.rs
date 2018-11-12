use std::f64::consts::LN_2;
use std::mem::size_of;

type Word = usize;
const BITS_PER_WORD: usize = size_of::<Word>() * 8;

pub struct Buckets {
    data: Vec<Word>,
    count: usize,
    bucket_size: u8,
    max: u8,
}

impl Buckets {
    pub fn with_fp_rate(items_count: usize, fp_rate: f64, bucket_size: u8) -> Self {
        Self::new(compute_m_num(items_count, fp_rate), bucket_size)
    }

    /// Creates a new Buckets with the provided number of buckets where
    /// each bucket is the specified number of bits.
    pub fn new(count: usize, bucket_size: u8) -> Self {
        assert!(bucket_size < 8);
        Self {
            data: vec![0; (count * bucket_size as usize + BITS_PER_WORD - 1) / BITS_PER_WORD],
            count,
            bucket_size,
            max: (1u8 << bucket_size) - 1,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn max_value(&self) -> u8 {
        self.max
    }

    pub fn reset(&mut self) {
        self.data.iter_mut().for_each(|x| *x = 0)
    }

    pub fn increment(&mut self, bucket: usize, delta: i8) {
        let v = self.get(bucket) as i8 + delta;
        let value = if v < 0 {
            0u8
        } else if v > self.max as i8 {
            self.max
        } else {
            v as u8
        };
        self.set(bucket, value);
    }

    pub fn set(&mut self, bucket: usize, byte: u8) {
        let offset = bucket * self.bucket_size as usize;
        let length = self.bucket_size as usize;
        let word = if byte > self.max as u8 { self.max } else { byte } as Word;
        self.set_word(offset, length, word);
    }

    pub fn get(&self, bucket: usize) -> u8 {
        self.get_word(bucket * self.bucket_size as usize, self.bucket_size as usize) as u8
    }

    fn set_word(&mut self, offset: usize, length: usize, word: Word) {
        let word_index = offset / BITS_PER_WORD;
        let word_offset = offset % BITS_PER_WORD;

        if word_offset + length > BITS_PER_WORD {
            let remain = BITS_PER_WORD - word_offset;
            self.set_word(offset, remain, word);
            self.set_word(offset + remain, length - remain, word >> remain);
        } else {
            let bit_mask = (1usize << length) - 1;
            self.data[word_index] &= !(bit_mask << word_offset);
            self.data[word_index] |= (word & bit_mask) << word_offset;
        }
    }

    fn get_word(&self, offset: usize, length: usize) -> Word {
        let word_index = offset / BITS_PER_WORD;
        let word_offset = offset % BITS_PER_WORD;
        if word_offset + length > BITS_PER_WORD {
            let remain = BITS_PER_WORD - word_offset;
            self.get_word(offset, remain) | (self.get_word(offset + remain, length - remain) << remain)
        } else {
            let bit_mask = (1usize << length) - 1;
            (self.data[word_index] & (bit_mask << word_offset)) >> word_offset
        }
    }
}

const LN_2_2: f64 = LN_2 * LN_2;

// Calculates the optimal buckets count, m, based on the number of
// items and the desired rate of false positives.
fn compute_m_num(items_count: usize, fp_rate: f64) -> usize {
    assert!(items_count > 0);
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    ((items_count as f64) * fp_rate.ln().abs() / LN_2_2).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_bit() {
        let mut buckets = Buckets::new(100, 1);
        buckets.set(0, 1);
        buckets.set(1, 0);
        buckets.set(2, 1);
        buckets.set(3, 0);
        assert_eq!(1, buckets.get(0));
        assert_eq!(0, buckets.get(1));
        assert_eq!(1, buckets.get(2));
        assert_eq!(0, buckets.get(3));
    }

    #[test]
    fn three_bits() {
        let mut buckets = Buckets::new(100, 3);
        buckets.set(0, 1);
        buckets.set(1, 2);
        buckets.set(10, 3);
        buckets.set(11, 4);
        buckets.set(20, 5);
        buckets.set(21, 6);
        assert_eq!(1, buckets.get(0));
        assert_eq!(2, buckets.get(1));
        assert_eq!(3, buckets.get(10));
        assert_eq!(4, buckets.get(11));
        assert_eq!(5, buckets.get(20));
        assert_eq!(6, buckets.get(21));
    }

    #[test]
    fn reset() {
        let mut buckets = Buckets::new(100, 1);
        buckets.set(1, 1);
        assert_eq!(1, buckets.get(1));
        buckets.reset();
        assert_eq!(0, buckets.get(1));
    }

    #[test]
    fn increment() {
        let mut buckets = Buckets::new(100, 3);
        buckets.increment(10, 2);
        assert_eq!(2, buckets.get(10));
        buckets.increment(10, 1);
        assert_eq!(3, buckets.get(10));
        buckets.increment(10, 100);
        assert_eq!(7, buckets.get(10));
        buckets.increment(10, -1);
        assert_eq!(6, buckets.get(10));
        buckets.increment(10, -10);
        assert_eq!(0, buckets.get(10));
    }
}
