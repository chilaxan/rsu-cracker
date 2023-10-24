// Does not work at all

pub const MULTIPLIER: u128 = 0x5DEECE66D;
pub const MULTIPLIER_INV: u128 = 0xDFE05BCB1365;
pub const ADDEND: u128 = 0xB;
pub const MASK: u128 = (1 << 48) - 1;
pub const GOLDEN_GAMMA: u128 = 0x9e3779b97f4a7c15;

pub struct JavaThreadLocalRandom {
    pub seed: u128,
    pub thread_id: u128
}

fn mix_murmur64(mut z: u128) -> u128 {
    z = (z ^ (z >> 33)).wrapping_mul(0xff51afd7ed558ccd);
    z = (z ^ (z >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
    z ^ (z >> 33)
}

impl JavaThreadLocalRandom {
    pub fn new(seed: u128, thread_id: u128) -> Self {
        Self { seed: mix_murmur64(seed) , thread_id }
    }

    pub fn new_raw(seed: u128, thread_id: u128) -> Self {
        Self { seed, thread_id }
    }

    pub fn next_seed(&mut self) -> u128 {
        let r = self.seed + (self.thread_id << 1) + GOLDEN_GAMMA;
        self.seed = r;
        return r;
    }

    fn mix32(&mut self, mut z: u128) -> u128 {
        z = (z ^ (z >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        (z ^ (z >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53) >> 32 as u32
    }

    pub fn next_int_unbound(&mut self) -> u128 {
        let tmp = self.next_seed();
        self.mix32(tmp) as u128
    }

    pub fn next(&mut self, bits: u128) -> u128 {
        self.next_int_unbound() >> (32 - bits)
    }

    pub fn next_int(&mut self, bound: u128) -> u128 {
        let mut r = self.next(31);
        let m = bound - 1;
        if bound & m == 0 {
            return (bound * r) >> 31;
        }
        loop {
            if (r as i64) - (r as i64).rem_euclid(bound as i64) + (m as i64) < 0 {
                r = self.next(31);
            } else {
                break;
            }
        }
        r % bound
    }
}
