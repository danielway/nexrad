//! CRC-32/BZIP2 (poly 0x04C11DB7, MSB-first, init/xorout 0xFFFFFFFF).
//!
//! * [`update`]: slice-by-16 over an output slice.
//! * [`run16`]: the register after `16 * q` copies of one byte, in O(q / 4)
//!   table steps. RLE1 fills make up most of the decoded bytes of radar
//!   data, so the expander checksums long runs with this instead of walking
//!   their bytes.

const POLY: u32 = 0x04C1_1DB7;

/// `T[s][b]`: register contribution of byte `b` followed by `s` zero bytes.
const fn make_all() -> [[u32; 256]; 64] {
    let mut t = [[0u32; 256]; 64];
    let mut i = 0usize;
    while i < 256 {
        let mut c = (i as u32) << 24;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x8000_0000 != 0 {
                (c << 1) ^ POLY
            } else {
                c << 1
            };
            k += 1;
        }
        t[0][i] = c;
        i += 1;
    }
    let mut s = 1usize;
    while s < 64 {
        let mut i = 0usize;
        while i < 256 {
            let prev = t[s - 1][i];
            t[s][i] = (prev << 8) ^ t[0][(prev >> 24) as usize];
            i += 1;
        }
        s += 1;
    }
    t
}

// Only read by the const fns below (compile time); never materialised.
#[allow(clippy::large_const_arrays)]
const ALL: [[u32; 256]; 64] = make_all();

const fn slice_tables() -> [[u32; 256]; 16] {
    let mut t = [[0u32; 256]; 16];
    let mut s = 0;
    while s < 16 {
        t[s] = ALL[s];
        s += 1;
    }
    t
}

/// Linear part of "64 bytes": T63..T60.
const fn z64_tables() -> [[u32; 256]; 4] {
    [ALL[60], ALL[61], ALL[62], ALL[63]]
}

/// `K[b] = XOR_{s < n} T[s][b]`: contribution of `n` trailing copies of `b`.
const fn k_table(n: usize) -> [u32; 256] {
    let mut k = [0u32; 256];
    let mut b = 0;
    while b < 256 {
        let mut x = 0u32;
        let mut s = 0;
        while s < n {
            x ^= ALL[s][b];
            s += 1;
        }
        k[b] = x;
        b += 1;
    }
    k
}

pub(crate) static TABLES: [[u32; 256]; 16] = slice_tables();
static Z64: [[u32; 256]; 4] = z64_tables();
static K12: [u32; 256] = k_table(12);
static K60: [u32; 256] = k_table(60);

/// Advance the (non-inverted) CRC register over `data`.
#[inline]
pub(crate) fn update(mut crc: u32, data: &[u8]) -> u32 {
    let t = &TABLES;
    let (chunks, remainder) = data.as_chunks::<16>();
    for c in chunks {
        let a = u32::from_be_bytes([c[0], c[1], c[2], c[3]]) ^ crc;
        crc = t[15][(a >> 24) as usize]
            ^ t[14][((a >> 16) & 0xff) as usize]
            ^ t[13][((a >> 8) & 0xff) as usize]
            ^ t[12][(a & 0xff) as usize]
            ^ t[11][c[4] as usize]
            ^ t[10][c[5] as usize]
            ^ t[9][c[6] as usize]
            ^ t[8][c[7] as usize]
            ^ t[7][c[8] as usize]
            ^ t[6][c[9] as usize]
            ^ t[5][c[10] as usize]
            ^ t[4][c[11] as usize]
            ^ t[3][c[12] as usize]
            ^ t[2][c[13] as usize]
            ^ t[1][c[14] as usize]
            ^ t[0][c[15] as usize];
    }
    for &b in remainder {
        crc = (crc << 8) ^ t[0][((crc >> 24) ^ b as u32) as usize];
    }
    crc
}

/// Advance the register over `16 * q` copies of byte `b`.
#[inline]
pub(crate) fn run16(mut crc: u32, b: u8, mut q: usize) -> u32 {
    let bb = (b as u32).wrapping_mul(0x0101_0101);
    let k60 = K60[b as usize];
    while q >= 4 {
        let a = crc ^ bb;
        crc = Z64[3][(a >> 24) as usize]
            ^ Z64[2][((a >> 16) & 0xff) as usize]
            ^ Z64[1][((a >> 8) & 0xff) as usize]
            ^ Z64[0][(a & 0xff) as usize]
            ^ k60;
        q -= 4;
    }
    let t = &TABLES;
    let k12 = K12[b as usize];
    while q > 0 {
        let a = crc ^ bb;
        crc = t[15][(a >> 24) as usize]
            ^ t[14][((a >> 16) & 0xff) as usize]
            ^ t[13][((a >> 8) & 0xff) as usize]
            ^ t[12][(a & 0xff) as usize]
            ^ k12;
        q -= 1;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitwise(data: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c ^= (b as u32) << 24;
            for _ in 0..8 {
                c = if c & 0x8000_0000 != 0 {
                    (c << 1) ^ POLY
                } else {
                    c << 1
                };
            }
        }
        !c
    }

    #[test]
    fn check_value() {
        assert_eq!(!update(!0, b"123456789"), 0xFC89_1918);
    }

    #[test]
    fn matches_bitwise_all_lengths() {
        let data: Vec<u8> = (0..300u32)
            .map(|i| (i.wrapping_mul(2654435761) >> 13) as u8)
            .collect();
        for n in 0..data.len() {
            assert_eq!(!update(!0, &data[..n]), bitwise(&data[..n]), "len {n}");
        }
    }

    #[test]
    fn run16_matches_bytes() {
        for &b in &[0u8, 1, 0x7f, 0x80, 0xff, 0x5a] {
            for q in 0..40usize {
                for &pre in &[0usize, 3, 16, 21] {
                    let prefix: Vec<u8> = (0..pre as u32).map(|i| (i * 37 + 11) as u8).collect();
                    let mut all = prefix.clone();
                    all.extend(std::iter::repeat_n(b, 16 * q));
                    let r = run16(update(!0, &prefix), b, q);
                    assert_eq!(!r, bitwise(&all), "b {b} q {q} pre {pre}");
                }
            }
        }
    }
}
