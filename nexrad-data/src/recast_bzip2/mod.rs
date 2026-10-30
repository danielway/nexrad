//! Vendored pure-Rust bzip2 decoder (the `recast-bzip2` backend).
//!
//! # Provenance
//!
//! This module is the decoder half of the `recast-radar-bzip2` crate, version
//! 0.1.3, from <https://github.com/recastsystems/recast-radar-tools>
//! (`crates/recast-radar-bzip2`), at git revision
//! `c206a2495c36341caa2a62ff7be3025320dbb028`. It is licensed under
//! `MIT OR Apache-2.0`; the MIT license text is reproduced in
//! `nexrad-data/THIRD_PARTY_NOTICES.md`. The crate is not published on
//! crates.io, so its sources are vendored here rather than pulled in as a
//! dependency.
//!
//! Changes from upstream:
//!
//! * `lib.rs` became this `mod.rs`; the encoder (`src/enc/`) and its public
//!   re-exports were dropped, and the crate docs were trimmed accordingly.
//! * The test-only `__set_check_crc` / `__seen_block_crcs` hooks and their
//!   plumbing were removed; CRCs are always verified.
//! * `Decoder::decode_two_into` is kept but marked `#[allow(dead_code)]`
//!   because `nexrad-data` does not use it yet.
//! * `block.rs` imports its siblings with `super::` instead of `crate::`.
//! * `crc::update` walks its input with `slice::as_chunks` instead of
//!   `chunks_exact(16)`, as clippy's `chunks_exact_to_as_chunks` asks.
//! * Formatted with this workspace's rustfmt (edition 2021).
//!
//! When updating, re-apply the above to a fresh copy of the upstream files.
//!
//! # Decompressing
//!
//! [`Decoder::decode_stream_into`] decodes one bzip2 stream (`BZh1`..`BZh9`
//! header, any number of blocks, end-of-stream marker) from a byte slice
//! and appends the decompressed bytes to a `Vec<u8>`.
//!
//! What is accepted and what is rejected follows libbzip2 1.0.8's normal
//! decompressor: a stream either decodes to exactly the bytes libbzip2
//! produces, or the call returns an [`Error`]. Block CRCs and the combined
//! stream CRC are checked. Randomised blocks (written by bzip2 0.9.0 and
//! earlier) decode, and streams with more than 18,002 selectors are clamped
//! the way libbzip2 clamps them. Bytes after the end-of-stream marker are
//! ignored, so a stream followed by trailing data, or by a second stream,
//! decodes its first stream only. Corrupt or truncated input never panics:
//! the worst case is an `Err`, after which the output vector is back at its
//! original length.
//!
//! # Limits
//!
//! The work buffers are fixed in size (see below); what untrusted input
//! controls is the output. One block can expand to about 47 MB (900,000
//! pre-RLE1 bytes, each 5-byte run standing for up to 259 bytes) and a
//! stream can hold any number of blocks, so decode untrusted input with a
//! bound: [`Decoder::set_max_output`] limits the bytes one call may append
//! (no limit by default). The limit is checked against a block's exact
//! decoded size before that block's output is allocated, and a stream that
//! would exceed it returns [`Error::OutputLimit`]. Block header fields
//! (origPtr, symbol map, group and selector counts) and Huffman code lengths
//! are validated before they are used ([`Error::BadBlockHeader`],
//! [`Error::BadHuffmanTables`]), and the selector count is clamped at 18,002
//! as in libbzip2.
//!
//! # Memory and reuse
//!
//! A [`Decoder`] owns the work buffers for one block: about 7 MiB of
//! zero-initialised address space, of which a block touches roughly five
//! bytes per pre-BWT symbol. They are allocated on the first decode and
//! reused by every later call, so keep one decoder per thread and reuse it.
//! Memory does not depend on the input: a call allocates nothing else but
//! its output.
//!
//! # Design
//!
//! Decompressing, per block:
//!
//! * A: a 64-bit MSB-first bit window and table-driven Huffman decoding
//!   (an 11-bit primary table per coding group whose entries carry the
//!   move-to-front index or a RUNA/RUNB/EOB kind, and a reference-semantics
//!   walk for longer codes), move-to-front through 16-byte window moves,
//!   RUNA/RUNB copies written as they are decoded, long runs recorded for
//!   stage B.
//! * B: write-only inverse-BWT vector build (`tt[j] = next << 12 | byte`),
//!   with the recorded runs filled as arithmetic sequences.
//! * C: dense pre-RLE1 byte chase into a byte buffer, the only
//!   latency-bound loop. [`Decoder::decode_two_into`] runs two of these
//!   chains in lockstep so their cache misses overlap.
//! * D: a SWAR RLE1 run search that gives the exact output size (checked
//!   against the output limit before allocating), expansion with short
//!   fixed-size copies and bulk fills, and the block CRC as slice-by-16 over
//!   contiguous output slices, with long fills advanced 16 or 64 bytes per
//!   table step.
//!
//! The block randomisation table in `rand.rs` is data from libbzip2's
//! `randtable.c` (BSD-style licence, notice kept in that file). Everything
//! else was written from the bzip2 format; libbzip2's `decompress.c` was read
//! for the exact acceptance rules.

mod bits;
mod block;
mod crc;
mod huff;
mod rand;

use bits::Bits;
use block::{BlockInfo, Workspace};

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const EOS_MAGIC: u64 = 0x1772_4538_5090;

/// Why a decode failed.
///
/// Any error means the input is not a valid bzip2 stream, or its output
/// exceeds the configured limit. On error the output vector is restored to
/// its original length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Missing or invalid `BZh1`..`BZh9` stream header.
    BadStreamHeader,
    /// Neither a block magic nor an end-of-stream magic where one is required.
    BadBlockMagic,
    /// Invalid block header fields (origPtr, symbol map, groups, selectors).
    BadBlockHeader,
    /// Invalid Huffman code lengths.
    BadHuffmanTables,
    /// Invalid Huffman, move-to-front or run-length data inside a block.
    BadBlockData,
    /// The input ended before the stream did.
    UnexpectedEof,
    /// A block's CRC does not match its decoded bytes.
    BlockCrcMismatch,
    /// The combined stream CRC does not match.
    StreamCrcMismatch,
    /// The decoded output would exceed the limit set by
    /// [`Decoder::set_max_output`].
    OutputLimit,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Error::BadStreamHeader => "bzip2: bad stream header",
            Error::BadBlockMagic => "bzip2: bad block magic",
            Error::BadBlockHeader => "bzip2: bad block header",
            Error::BadHuffmanTables => "bzip2: bad huffman tables",
            Error::BadBlockData => "bzip2: corrupt block data",
            Error::UnexpectedEof => "bzip2: unexpected end of input",
            Error::BlockCrcMismatch => "bzip2: block crc mismatch",
            Error::StreamCrcMismatch => "bzip2: stream crc mismatch",
            Error::OutputLimit => "bzip2: output limit exceeded",
        };
        f.write_str(s)
    }
}

impl std::error::Error for Error {}

/// A reusable bzip2 stream decoder.
///
/// Holds the block work buffers (see the module docs, *Memory and reuse*);
/// they are allocated on the first decode and reused afterwards. A second
/// set is allocated on the first [`Decoder::decode_two_into`] call. The
/// decoder is not shared between threads: keep one per thread.
pub struct Decoder {
    ws_a: Option<Box<Workspace>>,
    ws_b: Option<Box<Workspace>>,
    max_output: usize,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for Decoder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Decoder")
            .field("max_output", &self.max_output)
            .finish_non_exhaustive()
    }
}

/// Per-stream progress for the block loop.
struct Stream<'a> {
    bits: Bits<'a>,
    level: u32,
    combined: u32,
    start: usize,
}

enum Next {
    Block(BlockInfo),
    End,
}

impl<'a> Stream<'a> {
    fn begin(input: &'a [u8], out_len: usize) -> Result<Self, Error> {
        let Some(&level) = input.get(3) else {
            return Err(Error::BadStreamHeader);
        };
        if &input[..3] != b"BZh" || !(b'1'..=b'9').contains(&level) {
            return Err(Error::BadStreamHeader);
        }
        Ok(Stream {
            bits: Bits::new(input, 4),
            level: u32::from(level - b'0'),
            combined: 0,
            start: out_len,
        })
    }

    /// Read the next block magic and, for a block, stage A into `ws`.
    fn next(&mut self, ws: &mut Workspace) -> Result<Next, Error> {
        let magic = self.bits.read(48);
        if magic == BLOCK_MAGIC {
            block::read_block(&mut self.bits, self.level, ws).map(Next::Block)
        } else if magic == EOS_MAGIC {
            let stored = self.bits.read(32) as u32;
            if self.bits.overrun() {
                return Err(Error::UnexpectedEof);
            }
            if stored != self.combined {
                return Err(Error::StreamCrcMismatch);
            }
            Ok(Next::End)
        } else if self.bits.overrun() {
            Err(Error::UnexpectedEof)
        } else {
            Err(Error::BadBlockMagic)
        }
    }

    /// Stage D and CRC bookkeeping for a block whose chase is done.
    fn finish_block(
        &mut self,
        info: &BlockInfo,
        ws: &mut Workspace,
        out: &mut Vec<u8>,
        max_output: usize,
    ) -> Result<(), Error> {
        let n = info.nblock as usize;
        let Workspace { ll8, runs, .. } = ws;
        if info.randomised {
            block::derandomise(&mut ll8[..n]);
        }
        let limit = self.start.saturating_add(max_output);
        let crc = block::expand(ll8, n, runs, out, limit)?;
        if crc != info.stored_crc {
            return Err(Error::BlockCrcMismatch);
        }
        self.combined = self.combined.rotate_left(1) ^ crc;
        Ok(())
    }
}

/// One side of a paired decode: still running, or finished with a result.
enum Side<'a> {
    Running(Stream<'a>),
    Done(Result<(), Error>),
}

impl<'a> Side<'a> {
    fn begin(input: &'a [u8], out_len: usize) -> Self {
        match Stream::begin(input, out_len) {
            Ok(stream) => Side::Running(stream),
            Err(e) => Side::Done(Err(e)),
        }
    }

    /// Stage A of the next block, or finish this side at the end of its
    /// stream or on its first error.
    fn next_block(&mut self, ws: &mut Workspace) -> Option<BlockInfo> {
        let Side::Running(stream) = self else {
            return None;
        };
        match stream.next(ws) {
            Ok(Next::Block(info)) => Some(info),
            Ok(Next::End) => {
                *self = Side::Done(Ok(()));
                None
            }
            Err(e) => {
                *self = Side::Done(Err(e));
                None
            }
        }
    }

    fn finish_block(
        &mut self,
        info: &BlockInfo,
        ws: &mut Workspace,
        out: &mut Vec<u8>,
        max_output: usize,
    ) {
        let Side::Running(stream) = self else {
            return;
        };
        if let Err(e) = stream.finish_block(info, ws, out, max_output) {
            *self = Side::Done(Err(e));
        }
    }

    fn result(&self) -> Option<Result<(), Error>> {
        match self {
            Side::Running(_) => None,
            Side::Done(result) => Some(*result),
        }
    }
}

impl Decoder {
    /// A decoder with no output limit. The work buffers are allocated on
    /// the first decode.
    pub fn new() -> Self {
        Decoder {
            ws_a: None,
            ws_b: None,
            max_output: usize::MAX,
        }
    }

    /// Limit the number of bytes one call may append to its output vector.
    ///
    /// The limit applies to each stream of a call separately, counted from
    /// the vector's length when the call started. It is checked against a
    /// block's exact decoded size before that block's output is allocated,
    /// so exceeding it returns [`Error::OutputLimit`] without allocating
    /// the excess. `usize::MAX` (the default) means no limit.
    pub fn set_max_output(&mut self, limit: usize) {
        self.max_output = limit;
    }

    /// Decode one complete bzip2 stream from the start of `input`, appending
    /// the decompressed bytes to `out`.
    ///
    /// Bytes after the end-of-stream marker are ignored. On error `out` is
    /// truncated back to its original length. Never panics on any input.
    pub fn decode_stream_into(&mut self, input: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
        let start = out.len();
        let r = self.decode_one(input, out);
        if r.is_err() {
            out.truncate(start);
        }
        r
    }

    fn decode_one(&mut self, input: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
        let mut s = Stream::begin(input, out.len())?;
        let ws = self.ws_a.get_or_insert_with(Workspace::new);
        loop {
            match s.next(ws)? {
                Next::End => return Ok(()),
                Next::Block(info) => {
                    let n = info.nblock as usize;
                    block::build_tt(ws, n);
                    block::chase(ws, info.orig_ptr, n);
                    s.finish_block(&info, ws, out, self.max_output)?;
                }
            }
        }
    }

    /// Decode two independent streams in lockstep on the current thread.
    ///
    /// The result is the same as calling [`Decoder::decode_stream_into`] on
    /// `a` and then on `b`, including the output limit and the restoration
    /// of a failed side's vector; the two results are independent, so a
    /// corrupt side does not affect the other. Blocks are prepared for both
    /// streams, then their inverse-BWT chases run interleaved in one loop so
    /// the memory-latency-bound work overlaps. This costs about 1% more
    /// instructions than two single calls and allocates a second set of work
    /// buffers on first use; it pays off on large blocks.
    ///
    /// Not used by `nexrad-data` yet; kept so the module stays in step with
    /// upstream.
    #[allow(dead_code)]
    pub fn decode_two_into(
        &mut self,
        a: &[u8],
        out_a: &mut Vec<u8>,
        b: &[u8],
        out_b: &mut Vec<u8>,
    ) -> (Result<(), Error>, Result<(), Error>) {
        let start_a = out_a.len();
        let start_b = out_b.len();
        let max_output = self.max_output;
        let wa = self.ws_a.get_or_insert_with(Workspace::new);
        let wb = self.ws_b.get_or_insert_with(Workspace::new);

        let mut sa = Side::begin(a, start_a);
        let mut sb = Side::begin(b, start_b);
        let (ra, rb) = loop {
            let ia = sa.next_block(wa);
            let ib = sb.next_block(wb);
            match (&ia, &ib) {
                (Some(x), Some(y)) => {
                    block::build_tt(wa, x.nblock as usize);
                    block::build_tt(wb, y.nblock as usize);
                    block::chase_pair(
                        wa,
                        x.orig_ptr,
                        x.nblock as usize,
                        wb,
                        y.orig_ptr,
                        y.nblock as usize,
                    );
                }
                (Some(x), None) => {
                    block::build_tt(wa, x.nblock as usize);
                    block::chase(wa, x.orig_ptr, x.nblock as usize);
                }
                (None, Some(y)) => {
                    block::build_tt(wb, y.nblock as usize);
                    block::chase(wb, y.orig_ptr, y.nblock as usize);
                }
                (None, None) => {}
            }
            if let Some(x) = ia {
                sa.finish_block(&x, wa, out_a, max_output);
            }
            if let Some(y) = ib {
                sb.finish_block(&y, wb, out_b, max_output);
            }
            if let (Some(ra), Some(rb)) = (sa.result(), sb.result()) {
                break (ra, rb);
            }
        };
        if ra.is_err() {
            out_a.truncate(start_a);
        }
        if rb.is_err() {
            out_b.truncate(start_b);
        }
        (ra, rb)
    }
}
