//! MSB-first bit reader with a 64-bit left-aligned window.
//!
//! Invariant: the most significant bit of `buf` is stream bit
//! `pos * 8 - cnt`. Bits of `buf` below `cnt` are either zero or the
//! correct following stream bits, so the branch-free refill can OR a whole
//! big-endian word in at offset `cnt`. Past the end of the input the reader
//! supplies zero bits; callers detect that with [`Bits::overrun`] at the
//! checkpoints where a well-formed stream must still be inside its input.

pub(crate) struct Bits<'a> {
    pub data: &'a [u8],
    /// Next byte of `data` not yet OR-ed into `buf` (may exceed `data.len()`
    /// once zero padding has been supplied).
    pub pos: usize,
    pub buf: u64,
    pub cnt: u32,
}

impl<'a> Bits<'a> {
    #[inline]
    pub fn new(data: &'a [u8], pos: usize) -> Self {
        Bits {
            data,
            pos,
            buf: 0,
            cnt: 0,
        }
    }

    /// Ensure at least 56 valid bits.
    #[inline(always)]
    pub fn refill(&mut self) {
        if let Some(c) = self.data.get(self.pos..self.pos + 8) {
            let w = u64::from_be_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
            self.buf |= w >> self.cnt;
            self.pos += ((63 - self.cnt) >> 3) as usize;
            self.cnt |= 56;
        } else {
            self.refill_slow();
        }
    }

    #[cold]
    #[inline(never)]
    pub fn refill_slow(&mut self) {
        while self.cnt <= 56 {
            let b = self.data.get(self.pos).copied().unwrap_or(0);
            self.buf |= (b as u64) << (56 - self.cnt);
            self.pos += 1;
            self.cnt += 8;
        }
    }

    /// Read `n` bits, 1 <= n <= 56.
    #[inline]
    pub fn read(&mut self, n: u32) -> u64 {
        debug_assert!((1..=56).contains(&n));
        if self.cnt < n {
            self.refill();
        }
        let v = self.buf >> (64 - n);
        self.buf <<= n;
        self.cnt -= n;
        v
    }

    /// Stream bit offset of the next unread bit (u64: no overflow for inputs
    /// over 512 MiB on 32-bit targets).
    #[inline]
    pub fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.cnt as u64
    }

    /// True if more bits were consumed than the input holds.
    #[inline]
    pub fn overrun(&self) -> bool {
        self.bit_pos() > self.data.len() as u64 * 8
    }
}
