//! One bzip2 block: header + tables (stage A), inverse-BWT vector (stage B),
//! dense pre-RLE1 chase (stage C), RLE1 expansion + CRC (stage D).

use super::bits::Bits;
use super::crc;
use super::huff::{Huff, FAST_BITS, KIND_EOB};
use super::rand::RNUMS;
use super::Error;

/// tt / ll8 capacity: a power of two >= 900_000 (+ slack) so indices can be
/// masked instead of bounds-checked.
pub(crate) const TT_LEN: usize = 1 << 20;
pub(crate) const TT_MASK: usize = TT_LEN - 1;
const TT_SHIFT: u32 = 12;
/// Maximum RLE1 runs in one block (nblock_max / 5 = 180_000).
const RUNS_LEN: usize = 1 << 18;
/// RUNA/RUNB runs at least this long are recorded for stage B's bulk fill.
const ZRUN_MIN: usize = 16;
/// Capacity for recorded runs (nblock_max / (ZRUN_MIN + 1) < 2^17).
const ZRUNS_LEN: usize = 1 << 17;
pub(crate) const MAX_SELECTORS: usize = 18002;
const GROUP_SIZE: usize = 50;
/// Runs at least this long are checksummed with `crc::run16`.
const CRC_RUN_MIN: usize = 32;
/// MTF list is stored at `MTF_OFF..MTF_OFF + 256` with 16 bytes of slack on
/// both sides for the 16-byte window shift.
const MTF_OFF: usize = 16;

pub(crate) struct Workspace {
    /// Inverse-BWT vector: `tt[j] = (next << TT_SHIFT) | byte`. With
    /// `TT_SHIFT = 12`, `entry >> TT_SHIFT < TT_LEN` is provable, so the
    /// chase needs neither a mask nor a bounds check.
    pub tt: Box<[u32; TT_LEN]>,
    /// Stage A output (BWT last column), then reused for the stage C output
    /// (pre-RLE1 bytes in text order).
    pub ll8: Box<[u8; TT_LEN]>,
    /// Stage D: source offsets of RLE1 runs.
    pub runs: Box<[u32; RUNS_LEN]>,
    /// Stage A -> B: long RUNA/RUNB runs as `start << 32 | end`.
    pub zruns: Box<[u64; ZRUNS_LEN]>,
    pub nzruns: usize,
    pub selectors: Box<[u8; MAX_SELECTORS]>,
    pub tables: Box<[Huff; 6]>,
    pub counts: [u32; 256],
}

/// A zero-filled `[T; N]` on the heap, without building it on the stack.
fn boxed_zeroed<T: Copy + Default, const N: usize>() -> Box<[T; N]> {
    match Box::<[T; N]>::try_from(vec![T::default(); N].into_boxed_slice()) {
        Ok(array) => array,
        Err(_) => unreachable!("vec![T::default(); N] has length N"),
    }
}

impl Workspace {
    pub fn new() -> Box<Workspace> {
        Box::new(Workspace {
            tt: boxed_zeroed(),
            ll8: boxed_zeroed(),
            runs: boxed_zeroed(),
            zruns: boxed_zeroed(),
            nzruns: 0,
            selectors: boxed_zeroed(),
            tables: Box::new([
                Huff::EMPTY,
                Huff::EMPTY,
                Huff::EMPTY,
                Huff::EMPTY,
                Huff::EMPTY,
                Huff::EMPTY,
            ]),
            counts: [0; 256],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockInfo {
    pub stored_crc: u32,
    pub orig_ptr: u32,
    pub nblock: u32,
    pub randomised: bool,
}

/// Stage A: everything after the 48-bit block magic up to and including the
/// end-of-block symbol. Leaves the BWT last column in `ws.ll8[..nblock]` and
/// its byte histogram in `ws.counts`.
pub(crate) fn read_block(
    bits: &mut Bits<'_>,
    level: u32,
    ws: &mut Workspace,
) -> Result<BlockInfo, Error> {
    let stored_crc = bits.read(32) as u32;
    let randomised = bits.read(1) != 0;
    let orig_ptr = bits.read(24) as u32;
    if orig_ptr > 10 + 100_000 * level {
        return Err(Error::BadBlockHeader);
    }

    // Symbol map.
    let used16 = bits.read(16) as u32;
    let mut hot = Hot {
        mtf: [0u8; MTF_OFF + 256 + 16],
        counts: [0; 256],
    };
    let mtf = &mut hot.mtf;
    let mut n_in_use = 0usize;
    for i in 0..16u32 {
        if used16 & (0x8000 >> i) != 0 {
            let m = bits.read(16) as u32;
            for j in 0..16u32 {
                if m & (0x8000 >> j) != 0 {
                    mtf[MTF_OFF + n_in_use] = (i * 16 + j) as u8;
                    n_in_use += 1;
                }
            }
        }
    }
    if n_in_use == 0 {
        return Err(Error::BadBlockHeader);
    }
    let alpha = n_in_use + 2;

    let n_groups = bits.read(3) as usize;
    if !(2..=6).contains(&n_groups) {
        return Err(Error::BadBlockHeader);
    }
    let n_selectors = bits.read(15) as usize;
    if n_selectors < 1 {
        return Err(Error::BadBlockHeader);
    }
    // Selector MTF values (unary), undone against a small MTF list as we go.
    let mut pos = [0u8, 1, 2, 3, 4, 5];
    for i in 0..n_selectors {
        if bits.cnt < 8 {
            bits.refill();
        }
        let ones = (!bits.buf).leading_zeros() as usize;
        if ones >= n_groups {
            return Err(Error::BadBlockHeader);
        }
        bits.buf <<= ones + 1;
        bits.cnt -= ones as u32 + 1;
        if i < MAX_SELECTORS {
            let v = pos[ones];
            let mut k = ones;
            while k > 0 {
                pos[k] = pos[k - 1];
                k -= 1;
            }
            pos[0] = v;
            ws.selectors[i] = v;
        }
    }
    let n_sel = n_selectors.min(MAX_SELECTORS);

    // Coding tables (delta-coded lengths).
    let mut lens = [0u8; 258];
    for t in 0..n_groups {
        let mut curr = bits.read(5) as i32;
        for len in lens.iter_mut().take(alpha) {
            loop {
                if !(1..=20).contains(&curr) {
                    return Err(Error::BadHuffmanTables);
                }
                if bits.cnt < 2 {
                    bits.refill();
                }
                let two = bits.buf >> 62;
                if two < 2 {
                    // "0": end of this symbol's length.
                    bits.buf <<= 1;
                    bits.cnt -= 1;
                    break;
                }
                // "10" -> +1, "11" -> -1
                bits.buf <<= 2;
                bits.cnt -= 2;
                curr += if two == 2 { 1 } else { -1 };
            }
            *len = curr as u8;
        }
        ws.tables[t].build(&lens[..alpha]);
    }

    let nblock_max = 100_000 * level as usize;
    let nblock = decode_symbols(bits, ws, n_sel, nblock_max, &mut hot)?;
    if bits.overrun() {
        return Err(Error::UnexpectedEof);
    }
    if orig_ptr as usize >= nblock {
        return Err(Error::BadBlockData);
    }
    Ok(BlockInfo {
        stored_crc,
        orig_ptr,
        nblock: nblock as u32,
        randomised,
    })
}

/// Huffman + RUNA/RUNB + MTF decode into `ws.ll8`.
///
/// Semantics match the reference decoder, with two cheap reorderings that
/// cannot change the Ok/Err outcome or the bytes:
/// * run symbols write their copies immediately (the total is the same
///   bijective base-2 sum, and a run that overflows the block overflows at
///   its last symbol at the latest);
/// * the "literal past nblock_max" check is done once per 50-symbol group
///   and at EOB (the array has slack for the at most 50 extra bytes).
#[inline(never)]
fn decode_symbols(
    bits: &mut Bits<'_>,
    ws: &mut Workspace,
    n_sel: usize,
    nblock_max: usize,
    hot: &mut Hot,
) -> Result<usize, Error> {
    let Hot { mtf, counts } = hot;
    let data = bits.data;
    let mut buf = bits.buf;
    let mut cnt = bits.cnt as u8;
    // Unread input (bytes before it are already in `buf`); `pad` counts the
    // virtual zero bytes supplied past the end of the input.
    let mut rest: &[u8] = data.get(bits.pos..).unwrap_or(&[]);
    let mut pad = bits.pos.saturating_sub(data.len());
    let Workspace {
        ll8,
        selectors,
        tables,
        counts: ws_counts,
        zruns,
        nzruns,
        ..
    } = ws;
    let zruns: &mut [u64; ZRUNS_LEN] = zruns;
    let mut nz = 0usize;
    let mut run_start = 0usize;
    let mut last_rec = usize::MAX;
    let ll8: &mut [u8; TT_LEN] = ll8;
    let mut nblock = 0usize;
    let mut run_shift = 0u32;
    // nblock right after the last run symbol; a run symbol at any other
    // nblock starts a new run.
    let mut run_end = usize::MAX;
    let mut sel = 0usize;

    // MTF literal: move list[nn] to the front with 16-byte windows,
    // top-down (each window's source is still unmodified).
    macro_rules! literal {
        ($nn:expr) => {{
            let nn: usize = $nn;
            let v = mtf[MTF_OFF + nn];
            let mut k = nn;
            loop {
                // The slice is exactly 16 bytes, so `unwrap_or` never fires.
                let w: [u8; 16] = mtf[k..k + 16].try_into().unwrap_or([0; 16]);
                mtf[k + 1..k + 17].copy_from_slice(&w);
                if k <= MTF_OFF {
                    break;
                }
                k -= 16;
            }
            mtf[MTF_OFF] = v;
            ll8[nblock & TT_MASK] = v;
            counts[v as usize] += 1;
            nblock += 1;
        }};
    }

    let result = 'outer: loop {
        if nblock > nblock_max {
            break Err(Error::BadBlockData);
        }
        if sel >= n_sel {
            break Err(Error::BadBlockData);
        }
        let tab = &tables[selectors[sel] as usize % 6];
        let fast = &tab.fast;
        sel += 1;
        let mut left = GROUP_SIZE;
        loop {
            if cnt < 20 {
                if let Some(c) = rest.first_chunk::<8>() {
                    buf |= u64::from_be_bytes(*c) >> cnt;
                    rest = &rest[((63 - cnt) >> 3) as usize..];
                    cnt |= 56;
                } else {
                    let pos = data.len() - rest.len() + pad;
                    let mut b = Bits {
                        data,
                        pos,
                        buf,
                        cnt: cnt as u32,
                    };
                    b.refill_slow();
                    buf = b.buf;
                    cnt = b.cnt as u8;
                    rest = data.get(b.pos..).unwrap_or(&[]);
                    pad = b.pos.saturating_sub(data.len());
                }
            }
            let mut e = fast[(buf >> (64 - FAST_BITS)) as usize];
            let nn = (e >> 8) as usize;
            if nn != 0 {
                buf <<= (e & 0x3f) as u32;
                cnt = cnt.wrapping_sub(e as u8);
                literal!(nn);
            } else {
                if e == 0 {
                    e = tab.slow_entry(buf);
                    if e == 0 {
                        break 'outer Err(Error::BadBlockData);
                    }
                }
                let len = (e & 31) as u32;
                buf <<= len;
                cnt = cnt.wrapping_sub(len as u8);
                let kind = (e >> 5) & 7;
                if e >> 8 != 0 {
                    literal!((e >> 8) as usize);
                } else if kind == KIND_EOB {
                    if nblock > nblock_max {
                        break 'outer Err(Error::BadBlockData);
                    }
                    break 'outer Ok(nblock);
                } else {
                    if nblock != run_end {
                        run_shift = 0;
                        run_start = nblock;
                    }
                    if run_shift >= 21 {
                        break 'outer Err(Error::BadBlockData);
                    }
                    let add = (kind as usize) << run_shift;
                    run_shift += 1;
                    let end = nblock + add;
                    if end > nblock_max {
                        break 'outer Err(Error::BadBlockData);
                    }
                    let v = mtf[MTF_OFF];
                    counts[v as usize] += add as u32;
                    if add <= 16 {
                        let w = (v as u64).wrapping_mul(0x0101_0101_0101_0101).to_le_bytes();
                        ll8[nblock..nblock + 8].copy_from_slice(&w);
                        ll8[nblock + 8..nblock + 16].copy_from_slice(&w);
                    } else {
                        fill_run(&mut ll8[nblock..end], v);
                    }
                    nblock = end;
                    run_end = end;
                    if end - run_start >= ZRUN_MIN {
                        if last_rec != run_start {
                            last_rec = run_start;
                            nz += 1;
                        }
                        zruns[(nz - 1) & (ZRUNS_LEN - 1)] = ((run_start as u64) << 32) | end as u64;
                    }
                }
            }
            left -= 1;
            if left == 0 {
                break;
            }
        }
    };
    bits.buf = buf;
    bits.cnt = cnt as u32;
    bits.pos = data.len() - rest.len() + pad;
    *ws_counts = *counts;
    *nzruns = nz;
    result
}

/// MTF list and byte histogram side by side, so the decode loop addresses
/// both off one base register.
pub(crate) struct Hot {
    mtf: [u8; MTF_OFF + 256 + 16],
    counts: [u32; 256],
}

#[inline(never)]
fn fill_run(d: &mut [u8], v: u8) {
    d.fill(v);
}

/// Stage B: build `tt[j] = (i << TT_SHIFT) | L[i]` where `j` is the F-column slot of
/// L-position `i`. Write-only scatter (no read-modify-write of `tt`).
#[inline(never)]
pub(crate) fn build_tt(ws: &mut Workspace, nblock: usize) {
    let mut cftab = [0u32; 256];
    let mut sum = 0u32;
    for (c, &n) in cftab.iter_mut().zip(ws.counts.iter()) {
        *c = sum;
        sum += n;
    }
    let tt: &mut [u32; TT_LEN] = &mut ws.tt;
    let ll8: &[u8; TT_LEN] = &ws.ll8;
    #[inline(always)]
    fn scatter(tt: &mut [u32; TT_LEN], cftab: &mut [u32; 256], src: &[u8], mut iv: u32) {
        for &b in src {
            let c = &mut cftab[b as usize];
            let j = *c;
            *c = j + 1;
            tt[j as usize & TT_MASK] = iv | b as u32;
            iv = iv.wrapping_add(1 << TT_SHIFT);
        }
    }
    let mut i = 0usize;
    for &r in ws.zruns[..ws.nzruns.min(ZRUNS_LEN)].iter() {
        let s = (r >> 32) as usize;
        let e = (r as u32) as usize;
        scatter(tt, &mut cftab, &ll8[i..s], (i as u32) << TT_SHIFT);
        // A run of one byte value lands in consecutive slots of its bucket.
        let v = ll8[s & TT_MASK];
        let j0 = cftab[v as usize] as usize;
        let len = e - s;
        cftab[v as usize] += len as u32;
        let base = ((s as u32) << TT_SHIFT) | v as u32;
        for (k, t) in tt[j0..j0 + len].iter_mut().enumerate() {
            *t = base.wrapping_add((k as u32) << TT_SHIFT);
        }
        i = e;
    }
    scatter(tt, &mut cftab, &ll8[i..nblock], (i as u32) << TT_SHIFT);
}

/// Stage C: chase the cycle from `orig_ptr`, writing the pre-RLE1 bytes in
/// text order densely into `ll8[..nblock]`.
#[inline(never)]
pub(crate) fn chase(ws: &mut Workspace, orig_ptr: u32, nblock: usize) {
    let tt: &[u32; TT_LEN] = &ws.tt;
    let mut e = tt[orig_ptr as usize & TT_MASK];
    for slot in ws.ll8[..nblock].iter_mut() {
        *slot = e as u8;
        e = tt[(e >> TT_SHIFT) as usize];
    }
}

/// Stage C for two blocks in lockstep: the two dependency chains are
/// independent, so their cache misses can overlap on one core.
#[inline(never)]
pub(crate) fn chase_pair(
    wa: &mut Workspace,
    pa0: u32,
    na: usize,
    wb: &mut Workspace,
    pb0: u32,
    nb: usize,
) {
    let n = na.min(nb);
    let tta: &[u32; TT_LEN] = &wa.tt;
    let ttb: &[u32; TT_LEN] = &wb.tt;
    let (da, ra) = wa.ll8[..na].split_at_mut(n);
    let (db, rb) = wb.ll8[..nb].split_at_mut(n);
    let mut ea = tta[pa0 as usize & TT_MASK];
    let mut eb = ttb[pb0 as usize & TT_MASK];
    for (sa, sb) in da.iter_mut().zip(db.iter_mut()) {
        *sa = ea as u8;
        *sb = eb as u8;
        ea = tta[(ea >> TT_SHIFT) as usize];
        eb = ttb[(eb >> TT_SHIFT) as usize];
    }
    for slot in ra.iter_mut() {
        *slot = ea as u8;
        ea = tta[(ea >> TT_SHIFT) as usize];
    }
    for slot in rb.iter_mut() {
        *slot = eb as u8;
        eb = ttb[(eb >> TT_SHIFT) as usize];
    }
}

/// Apply the legacy block randomisation mask to the pre-RLE1 bytes.
#[cold]
pub(crate) fn derandomise(ll8: &mut [u8]) {
    let mut to_go: u32 = 0;
    let mut t_pos = 0usize;
    for b in ll8.iter_mut() {
        if to_go == 0 {
            to_go = u32::from(RNUMS[t_pos]);
            t_pos = (t_pos + 1) & 511;
        }
        to_go -= 1;
        if to_go == 1 {
            *b ^= 1;
        }
    }
}

/// Bit k of the result is set iff byte k of `a` equals byte k of `b`.
#[inline(always)]
fn eq_bits(a: u64, b: u64) -> u32 {
    const LO7: u64 = 0x7F7F_7F7F_7F7F_7F7F;
    let d = a ^ b;
    let z = !(((d & LO7).wrapping_add(LO7)) | d | LO7);
    ((z >> 7).wrapping_mul(0x0102_0408_1020_4080) >> 56) as u32
}

/// The first 8 bytes of `s` as a little-endian word (`s.len() >= 8`).
#[inline(always)]
fn le64(s: &[u8]) -> u64 {
    let mut w = [0u8; 8];
    w.copy_from_slice(&s[..8]);
    u64::from_le_bytes(w)
}

/// Stage D pass 1: locate RLE1 runs (4 equal bytes + count byte) and return
/// (number of runs, exact output length).
#[inline(never)]
fn find_runs(src: &[u8], runs: &mut [u32; RUNS_LEN]) -> Result<(usize, usize), Error> {
    let n = src.len();
    let mut nr = 0usize;
    let mut extra = 0usize;
    let mut i = 0usize;
    while i + 17 <= n {
        let a = &src[i..i + 17];
        let w0 = le64(a);
        let w0n = le64(&a[1..]);
        let w1 = le64(&a[8..]);
        let w1n = le64(&a[9..]);
        let m = eq_bits(w0, w0n) | (eq_bits(w1, w1n) << 8);
        let s = m & (m >> 1) & (m >> 2) & 0x3FFF;
        if s == 0 {
            i += 14;
            continue;
        }
        let rp = i + s.trailing_zeros() as usize;
        let Some(&count) = src.get(rp + 4) else {
            return Err(Error::BadBlockData);
        };
        runs[nr & (RUNS_LEN - 1)] = rp as u32;
        nr += 1;
        extra += count as usize;
        i = rp + 5;
    }
    while i + 3 < n {
        let b = src[i];
        if src[i + 1] == b && src[i + 2] == b && src[i + 3] == b {
            let Some(&count) = src.get(i + 4) else {
                return Err(Error::BadBlockData);
            };
            runs[nr & (RUNS_LEN - 1)] = i as u32;
            nr += 1;
            extra += count as usize;
            i += 5;
        } else {
            i += 1;
        }
    }
    // Each run turns 5 source bytes into 4 + count output bytes.
    Ok((nr, n - nr + extra))
}

/// Stage D: RLE1 expansion of the pre-RLE1 bytes `ll8[..n]` onto `out`
/// (exactly reserved), with the block CRC computed over contiguous output
/// slices and long fills checksummed arithmetically. `limit` bounds
/// `out.len()`.
#[inline(never)]
pub(crate) fn expand(
    ll8: &[u8; TT_LEN],
    n: usize,
    runs: &mut [u32; RUNS_LEN],
    out_vec: &mut Vec<u8>,
    limit: usize,
) -> Result<u32, Error> {
    let (nr, out_len) = find_runs(&ll8[..n], runs)?;
    if out_vec.len().saturating_add(out_len) > limit {
        return Err(Error::OutputLimit);
    }
    // Work on a local Vec so its pointer/len/cap live in registers.
    let mut out = std::mem::take(out_vec);
    out.reserve(out_len + 32);
    let mut crc = !0u32;
    let mut crc_pos = out.len();
    let mut lit = 0usize;
    for &rp in runs[..nr].iter() {
        let rp = rp as usize & TT_MASK;
        let l = rp.wrapping_sub(lit);
        if l <= 16 {
            // Short literal: copy a whole 16-byte chunk, keep `l` bytes.
            let at = lit & TT_MASK;
            let keep = out.len() + l;
            out.extend_from_slice(&ll8[at..at + 16]);
            out.truncate(keep);
        } else {
            out.extend_from_slice(&ll8[lit..rp]);
        }
        let b = ll8[rp];
        let rl = 4 + ll8[(rp + 4) & TT_MASK] as usize;
        let run_start = out.len();
        if rl <= 16 {
            out.extend_from_slice(&[b; 16]);
            out.truncate(run_start + rl);
        } else {
            out.resize(run_start + rl, b);
        }
        if rl >= CRC_RUN_MIN {
            // Checksum everything up to the run plus enough run bytes to
            // reach a 16-byte boundary, then the rest of the run in whole
            // 16-byte steps; the < 16 leftover run bytes start the next slice.
            let pad = (16 - (run_start - crc_pos) % 16) % 16;
            crc = crc::update(crc, &out[crc_pos..run_start + pad]);
            let q = (rl - pad) / 16;
            crc = crc::run16(crc, b, q);
            crc_pos = run_start + pad + 16 * q;
        }
        lit = rp + 5;
    }
    out.extend_from_slice(&ll8[lit..n]);
    crc = crc::update(crc, &out[crc_pos..]);
    *out_vec = out;
    Ok(!crc)
}
