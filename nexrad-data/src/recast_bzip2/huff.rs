//! Canonical Huffman decoding tables for bzip2 coding groups.
//!
//! bzip2 assigns codes canonically by (length, symbol). A decoder that
//! follows the reference semantics accepts *any* length set with lengths in
//! 1..=20, including over-full and incomplete codes: it walks lengths from
//! the shortest upward and takes the first length `l` whose `l`-bit prefix
//! `p` satisfies `p <= limit[l]`, then emits `perm[p - base[l]]`.
//!
//! This table layer reproduces those semantics exactly:
//! * `fast` is indexed by the next `FAST_BITS` bits. An entry holds
//!   `map(symbol) | length` for every window whose first match has
//!   `length <= FAST_BITS`; 0 means "no match that short". Layout: bits 0-4
//!   length; for MTF literals bits 8-15 hold the MTF index (`symbol - 1`,
//!   never 0) and bits 5-7 are zero, so `entry >> 8 != 0` identifies a
//!   literal and the low byte is exactly the shift; for RUNA / RUNB / EOB
//!   bits 8-15 are zero and bits 5-7 hold kind 1 / 2 / 3 (kind 0 = slow).
//! * `limit` / `base` / `perm` drive the slow walk for longer codes (and the
//!   error case where no length up to 20 matches).

pub(crate) const FAST_BITS: u32 = 11;
pub(crate) const FAST_SIZE: usize = 1 << FAST_BITS;
pub(crate) const MAX_CODE_LEN: usize = 20;
pub(crate) const KIND_RUNA: u16 = 1;
pub(crate) const KIND_RUNB: u16 = 2;
pub(crate) const KIND_EOB: u16 = 3;

/// Entry payload for decoded symbol `sym` (see module docs).
#[inline]
pub(crate) fn map_symbol(sym: u32, eob: u32) -> u16 {
    if sym == 0 {
        KIND_RUNA << 5
    } else if sym == 1 {
        KIND_RUNB << 5
    } else if sym == eob {
        KIND_EOB << 5
    } else {
        ((sym - 1) << 8) as u16
    }
}

#[derive(Clone)]
pub(crate) struct Huff {
    pub fast: [u16; FAST_SIZE],
    pub limit: [i32; 22],
    pub base: [i32; 22],
    pub perm: [u16; 258],
    pub min_len: u32,
    pub alpha: u32,
    pub eob: u32,
}

impl Huff {
    pub const EMPTY: Huff = Huff {
        fast: [0; FAST_SIZE],
        limit: [0; 22],
        base: [0; 22],
        perm: [0; 258],
        min_len: 1,
        alpha: 0,
        eob: 0,
    };

    #[allow(clippy::needless_range_loop)]
    /// Build from code lengths (each already validated to 1..=20).
    /// `lens.len()` is the alphabet size; its last symbol is EOB.
    pub fn build(&mut self, lens: &[u8]) {
        let alpha = lens.len();
        debug_assert!((3..=258).contains(&alpha));
        self.eob = alpha as u32 - 1;
        let mut count = [0i32; 22];
        let mut min_len = 32usize;
        let mut max_len = 0usize;
        for &l in lens {
            let l = l as usize;
            count[l] += 1;
            min_len = min_len.min(l);
            max_len = max_len.max(l);
        }
        // offs[l] = number of symbols with length < l.
        let mut offs = [0i32; 22];
        for l in 1..22 {
            offs[l] = offs[l - 1] + count[l - 1];
        }
        let mut next = offs;
        for (s, &l) in lens.iter().enumerate() {
            let slot = &mut next[l as usize];
            self.perm[*slot as usize] = s as u16;
            *slot += 1;
        }
        self.limit = [0; 22];
        self.base = offs;
        let mut vec = 0i32;
        for l in min_len..=max_len {
            vec += count[l];
            self.limit[l] = vec - 1;
            vec <<= 1;
        }
        for l in min_len + 1..=max_len {
            self.base[l] = ((self.limit[l - 1] + 1) << 1) - offs[l];
        }
        self.min_len = min_len as u32;
        self.alpha = alpha as u32;

        self.fast = [0; FAST_SIZE];
        let top = max_len.min(FAST_BITS as usize);
        for l in min_len..=top {
            let first = if l == min_len {
                0
            } else {
                (self.limit[l - 1] + 1) << 1
            };
            let shift = FAST_BITS as usize - l;
            for k in 0..count[l] {
                let code = first + k;
                if code >= (1 << l) {
                    break;
                }
                let sym = self.perm[(offs[l] + k) as usize];
                let entry = map_symbol(sym as u32, self.eob) | l as u16;
                let lo = (code as usize) << shift;
                let hi = (code as usize + 1) << shift;
                self.fast[lo..hi].fill(entry);
            }
        }
    }

    /// Slow walk as a mapped entry (0 = invalid code).
    #[cold]
    #[inline(never)]
    pub fn slow_entry(&self, window: u64) -> u16 {
        match self.decode_slow(window) {
            Some((sym, len)) => map_symbol(sym, self.eob) | len as u16,
            None => 0,
        }
    }

    /// Reference-semantics walk for windows the fast table does not resolve.
    /// `window` holds at least 20 valid bits at its top.
    #[inline]
    pub fn decode_slow(&self, window: u64) -> Option<(u32, u32)> {
        let mut zn = (self.min_len as usize).max(FAST_BITS as usize + 1);
        while zn <= MAX_CODE_LEN {
            let zvec = (window >> (64 - zn)) as i32;
            if zvec <= self.limit[zn] {
                let idx = zvec - self.base[zn];
                if idx < 0 || idx as u32 >= self.alpha {
                    return None;
                }
                return Some((self.perm[idx as usize] as u32, zn as u32));
            }
            zn += 1;
        }
        None
    }

    /// Full decode as a mapped entry (0 = invalid). Test helper.
    #[cfg(test)]
    pub fn entry(&self, window: u64) -> u16 {
        let e = self.fast[(window >> (64 - FAST_BITS)) as usize];
        if e != 0 {
            e
        } else {
            self.slow_entry(window)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Literal transcription of the reference decode walk.
    fn reference(lens: &[u8], window: u64) -> Option<(u32, u32)> {
        let alpha = lens.len();
        let min_len = *lens.iter().min().unwrap() as usize;
        let max_len = *lens.iter().max().unwrap() as usize;
        let mut perm = vec![0u32; 258];
        let mut pp = 0;
        for i in min_len..=max_len {
            for (j, &l) in lens.iter().enumerate() {
                if l as usize == i {
                    perm[pp] = j as u32;
                    pp += 1;
                }
            }
        }
        let mut base = [0i32; 23];
        for &l in lens {
            base[l as usize + 1] += 1;
        }
        for i in 1..23 {
            base[i] += base[i - 1];
        }
        let mut limit = [0i32; 23];
        let mut vec = 0;
        for i in min_len..=max_len {
            vec += base[i + 1] - base[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_len + 1..=max_len {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        let mut zn = min_len;
        loop {
            if zn > 20 {
                return None;
            }
            let zvec = (window >> (64 - zn)) as i32;
            if zvec <= limit[zn] {
                let idx = zvec - base[zn];
                if !(0..258).contains(&idx) {
                    return None;
                }
                assert!(
                    (idx as usize) < alpha,
                    "reference would read uninitialised perm"
                );
                return Some((perm[idx as usize], zn as u32));
            }
            zn += 1;
        }
    }

    fn expect(lens: &[u8], w: u64) -> u16 {
        reference(lens, w).map_or(0, |(s, l)| map_symbol(s, lens.len() as u32 - 1) | l as u16)
    }

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }

    #[test]
    fn matches_reference_on_random_length_sets() {
        let mut rng = Rng(12345);
        let mut h = Huff::EMPTY;
        for iter in 0..3000 {
            let alpha = 3 + (rng.next() % 256) as usize;
            let mode = iter % 4;
            let lens: Vec<u8> = (0..alpha)
                .map(|_| match mode {
                    0 => 1 + (rng.next() % 20) as u8,
                    1 => 1 + (rng.next() % 4) as u8,
                    2 => 8 + (rng.next() % 13) as u8,
                    _ => 9 + (rng.next() % 3) as u8,
                })
                .collect();
            h.build(&lens);
            for _ in 0..400 {
                let w = rng.next();
                let w = match rng.next() % 4 {
                    0 => w | (0xFFFFu64 << 48),
                    1 => w & !(0xFFFFu64 << 48),
                    _ => w,
                };
                assert_eq!(h.entry(w), expect(&lens, w), "lens {lens:?} window {w:#x}");
            }
        }
    }

    #[test]
    fn exhaustive_prefixes_small_alphabets() {
        let mut rng = Rng(777);
        let mut h = Huff::EMPTY;
        for _ in 0..300 {
            let alpha = 3 + (rng.next() % 6) as usize;
            let lens: Vec<u8> = (0..alpha).map(|_| 1 + (rng.next() % 12) as u8).collect();
            h.build(&lens);
            for top in 0..(1u64 << 14) {
                let w = (top << 50) | (rng.next() >> 14);
                assert_eq!(h.entry(w), expect(&lens, w), "lens {lens:?} window {w:#x}");
            }
        }
    }
}
