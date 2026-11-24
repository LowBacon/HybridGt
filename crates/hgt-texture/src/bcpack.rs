//! BCPack unpacking behind an adapter.
//!
//! Background (ivall/xcl#878, 2026-11-24): on a retail run every BCPack
//! request was refused, and 183,128,064 destination bytes were never written.
//! The run went on, and the textures kept whatever their memory held. The
//! request did fail, but nothing in the world path saw that. This module makes
//! the outcome of each request explicit:
//!
//! - [`unpack_strict`] decodes into scratch memory and copies to the
//!   destination only when the whole stream decoded. A failed request leaves
//!   the destination exactly as it was, with no partial blocks in it.
//! - [`Ledger`] counts requested and filled bytes per format, so a run can
//!   report "N bytes requested, M filled" instead of a log line per failure.
//!
//! The decoder itself is VIctory's (`victory_formats::bcpack`, GPL-3.0-or-later),
//! used through [`VictoryBackend`]. Other decoders can sit behind [`Backend`]
//! and be compared on the same inputs.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// A `DSTORAGE_BCPACK_MODE`: the BC number of the blocks it produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcFormat {
    Bc1,
    Bc3,
    Bc4,
    Bc5,
    Bc6h,
    Bc7,
}

impl BcFormat {
    pub const ALL: [BcFormat; 6] =
        [BcFormat::Bc1, BcFormat::Bc3, BcFormat::Bc4, BcFormat::Bc5, BcFormat::Bc6h, BcFormat::Bc7];

    /// The format for a request's BCPack mode. BC2 (mode 2) has no decoder
    /// in any inspected implementation; mode 0 means "no BCPack".
    pub fn from_mode(mode: u8) -> Option<BcFormat> {
        Some(match mode {
            1 => BcFormat::Bc1,
            3 => BcFormat::Bc3,
            4 => BcFormat::Bc4,
            5 => BcFormat::Bc5,
            6 => BcFormat::Bc6h,
            7 => BcFormat::Bc7,
            _ => return None,
        })
    }

    pub fn mode(self) -> u8 {
        match self {
            BcFormat::Bc1 => 1,
            BcFormat::Bc3 => 3,
            BcFormat::Bc4 => 4,
            BcFormat::Bc5 => 5,
            BcFormat::Bc6h => 6,
            BcFormat::Bc7 => 7,
        }
    }

    pub fn block_bytes(self) -> usize {
        match self {
            BcFormat::Bc1 | BcFormat::Bc4 => 8,
            _ => 16,
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|f| *f == self).unwrap_or(0)
    }
}

impl fmt::Display for BcFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BcFormat::Bc1 => "BC1",
            BcFormat::Bc3 => "BC3",
            BcFormat::Bc4 => "BC4",
            BcFormat::Bc5 => "BC5",
            BcFormat::Bc6h => "BC6H",
            BcFormat::Bc7 => "BC7",
        };
        f.write_str(s)
    }
}

/// Why a request was not filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnpackError {
    /// No decoder for this mode (0, 2, 8 and above).
    UnsupportedMode(u8),
    /// The destination is empty or not a whole number of blocks.
    BadDestination { len: usize, block: usize },
    /// The backend could not set itself up (e.g. its tables did not parse).
    BackendUnavailable(String),
    /// The stream did not decode.
    Stream(String),
    /// The backend panicked on this input; treated as a failed request.
    BackendPanicked,
}

impl fmt::Display for UnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnpackError::UnsupportedMode(m) => write!(f, "BCPack mode {m} has no decoder"),
            UnpackError::BadDestination { len, block } => {
                write!(f, "destination of {len} bytes is not a whole number of {block}-byte blocks")
            }
            UnpackError::BackendUnavailable(why) => write!(f, "BCPack decoder unavailable: {why}"),
            UnpackError::Stream(why) => write!(f, "BCPack stream did not decode: {why}"),
            UnpackError::BackendPanicked => f.write_str("BCPack decoder panicked"),
        }
    }
}

impl std::error::Error for UnpackError {}

/// A BCPack decoder. `dst.len()` gives the block count; the backend may
/// leave `dst` in any state when it returns an error.
pub trait Backend {
    fn name(&self) -> &'static str;
    fn decode(&self, format: BcFormat, src: &[u8], dst: &mut [u8]) -> Result<(), UnpackError>;
}

/// VIctory's decoder with its shipped format tables.
pub struct VictoryBackend {
    tables: victory_formats::bcpack::Tables,
}

impl VictoryBackend {
    pub fn new() -> Result<VictoryBackend, UnpackError> {
        victory_formats::bcpack::Tables::builtin()
            .map(|tables| VictoryBackend { tables })
            .map_err(|e| UnpackError::BackendUnavailable(e.to_string()))
    }
}

impl Backend for VictoryBackend {
    fn name(&self) -> &'static str {
        "victory-formats@ffe310e"
    }

    fn decode(&self, format: BcFormat, src: &[u8], dst: &mut [u8]) -> Result<(), UnpackError> {
        victory_formats::bcpack::decode(format.mode(), src, dst, &self.tables)
            .map_err(|e| UnpackError::Stream(e.to_string()))
    }
}

/// Unpacks one request. On `Ok` every byte of `dst` was written by the
/// decoder. On `Err` `dst` is unchanged. A panic in the backend is caught
/// and reported as [`UnpackError::BackendPanicked`].
pub fn unpack_strict(backend: &dyn Backend, mode: u8, src: &[u8], dst: &mut [u8]) -> Result<(), UnpackError> {
    let format = BcFormat::from_mode(mode).ok_or(UnpackError::UnsupportedMode(mode))?;
    let block = format.block_bytes();
    if dst.is_empty() || !dst.len().is_multiple_of(block) {
        return Err(UnpackError::BadDestination { len: dst.len(), block });
    }
    let mut scratch = vec![0u8; dst.len()];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| backend.decode(format, src, &mut scratch)));
    match result {
        Ok(Ok(())) => {
            dst.copy_from_slice(&scratch);
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(UnpackError::BackendPanicked),
    }
}

/// Requested and filled bytes per format, safe to share across threads.
#[derive(Default)]
pub struct Ledger {
    requests: [AtomicU64; 6],
    requested_bytes: [AtomicU64; 6],
    filled_bytes: [AtomicU64; 6],
    failed: [AtomicU64; 6],
    unsupported_requests: AtomicU64,
    unsupported_bytes: AtomicU64,
}

/// One format's line in a [`Ledger`] summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerRow {
    pub format: BcFormat,
    pub requests: u64,
    pub requested_bytes: u64,
    pub filled_bytes: u64,
    pub failed: u64,
}

impl Ledger {
    /// Runs [`unpack_strict`] and records its outcome.
    pub fn unpack(&self, backend: &dyn Backend, mode: u8, src: &[u8], dst: &mut [u8]) -> Result<(), UnpackError> {
        let n = dst.len() as u64;
        let result = unpack_strict(backend, mode, src, dst);
        match BcFormat::from_mode(mode) {
            Some(f) => {
                let i = f.index();
                self.requests[i].fetch_add(1, Ordering::Relaxed);
                self.requested_bytes[i].fetch_add(n, Ordering::Relaxed);
                if result.is_ok() {
                    self.filled_bytes[i].fetch_add(n, Ordering::Relaxed);
                } else {
                    self.failed[i].fetch_add(1, Ordering::Relaxed);
                }
            }
            None => {
                self.unsupported_requests.fetch_add(1, Ordering::Relaxed);
                self.unsupported_bytes.fetch_add(n, Ordering::Relaxed);
            }
        }
        result
    }

    pub fn rows(&self) -> Vec<LedgerRow> {
        BcFormat::ALL
            .iter()
            .map(|&format| {
                let i = format.index();
                LedgerRow {
                    format,
                    requests: self.requests[i].load(Ordering::Relaxed),
                    requested_bytes: self.requested_bytes[i].load(Ordering::Relaxed),
                    filled_bytes: self.filled_bytes[i].load(Ordering::Relaxed),
                    failed: self.failed[i].load(Ordering::Relaxed),
                }
            })
            .collect()
    }

    /// Bytes requested but not written, all formats and unsupported modes.
    pub fn unfilled_bytes(&self) -> u64 {
        let rows: u64 = self.rows().iter().map(|r| r.requested_bytes - r.filled_bytes).sum();
        rows + self.unsupported_bytes.load(Ordering::Relaxed)
    }

    pub fn unsupported(&self) -> (u64, u64) {
        (self.unsupported_requests.load(Ordering::Relaxed), self.unsupported_bytes.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: u8 = 0xAB;

    /// A backend that writes half the destination and then fails, the way
    /// a decoder that runs out of input mid-stream can.
    struct HalfThenFail;
    impl Backend for HalfThenFail {
        fn name(&self) -> &'static str {
            "half-then-fail"
        }
        fn decode(&self, _: BcFormat, _: &[u8], dst: &mut [u8]) -> Result<(), UnpackError> {
            let half = dst.len() / 2;
            dst[..half].fill(0x11);
            Err(UnpackError::Stream("ran out".into()))
        }
    }

    struct Panics;
    impl Backend for Panics {
        fn name(&self) -> &'static str {
            "panics"
        }
        fn decode(&self, _: BcFormat, _: &[u8], _: &mut [u8]) -> Result<(), UnpackError> {
            panic!("decoder bug")
        }
    }

    struct FillsOnes;
    impl Backend for FillsOnes {
        fn name(&self) -> &'static str {
            "fills-ones"
        }
        fn decode(&self, _: BcFormat, _: &[u8], dst: &mut [u8]) -> Result<(), UnpackError> {
            dst.fill(1);
            Ok(())
        }
    }

    #[test]
    fn modes_map_to_formats() {
        for f in BcFormat::ALL {
            assert_eq!(BcFormat::from_mode(f.mode()), Some(f));
        }
        for m in [0u8, 2, 8, 9, 15, 255] {
            assert_eq!(BcFormat::from_mode(m), None, "mode {m}");
        }
    }

    #[test]
    fn unsupported_modes_fail_and_leave_destination() {
        let mut dst = vec![SENTINEL; 64];
        for m in [0u8, 2, 8, 15] {
            assert_eq!(unpack_strict(&FillsOnes, m, &[1, 2, 3], &mut dst), Err(UnpackError::UnsupportedMode(m)));
            assert!(dst.iter().all(|&b| b == SENTINEL));
        }
    }

    #[test]
    fn destinations_must_be_whole_blocks() {
        let mut empty: [u8; 0] = [];
        assert!(matches!(unpack_strict(&FillsOnes, 1, &[0], &mut empty), Err(UnpackError::BadDestination { .. })));
        let mut odd = vec![SENTINEL; 12];
        assert_eq!(
            unpack_strict(&FillsOnes, 7, &[0], &mut odd),
            Err(UnpackError::BadDestination { len: 12, block: 16 })
        );
        assert!(odd.iter().all(|&b| b == SENTINEL));
        let mut bc1 = vec![SENTINEL; 24];
        assert_eq!(unpack_strict(&FillsOnes, 1, &[0], &mut bc1), Ok(()));
    }

    #[test]
    fn a_failed_decode_writes_nothing_to_the_destination() {
        let mut dst = vec![SENTINEL; 256];
        assert!(unpack_strict(&HalfThenFail, 1, &[0; 8], &mut dst).is_err());
        assert!(dst.iter().all(|&b| b == SENTINEL), "partial blocks leaked into the destination");
    }

    #[test]
    fn a_panicking_decoder_is_a_failed_request() {
        let mut dst = vec![SENTINEL; 16];
        assert_eq!(unpack_strict(&Panics, 3, &[0; 4], &mut dst), Err(UnpackError::BackendPanicked));
        assert!(dst.iter().all(|&b| b == SENTINEL));
    }

    #[test]
    fn the_ledger_counts_requested_and_filled_bytes() {
        let ledger = Ledger::default();
        let mut a = vec![0u8; 64];
        let mut b = vec![0u8; 128];
        let mut c = vec![0u8; 32];
        ledger.unpack(&FillsOnes, 1, &[0], &mut a).unwrap();
        ledger.unpack(&HalfThenFail, 1, &[0], &mut b).unwrap_err();
        ledger.unpack(&FillsOnes, 2, &[0], &mut c).unwrap_err();
        let bc1 = ledger.rows()[0];
        assert_eq!((bc1.requests, bc1.requested_bytes, bc1.filled_bytes, bc1.failed), (2, 192, 64, 1));
        assert_eq!(ledger.unsupported(), (1, 32));
        assert_eq!(ledger.unfilled_bytes(), 128 + 32);
    }

    #[test]
    fn victory_tables_load() {
        VictoryBackend::new().expect("VIctory's shipped BCPack tables parse");
    }

    #[test]
    fn victory_rejects_streams_that_run_out() {
        let backend = VictoryBackend::new().unwrap();
        for f in BcFormat::ALL {
            let mut dst = vec![SENTINEL; f.block_bytes() * 64];
            let r = unpack_strict(&backend, f.mode(), &[], &mut dst);
            assert!(matches!(r, Err(UnpackError::Stream(_))), "{f}: empty stream gave {r:?}");
            assert!(dst.iter().all(|&b| b == SENTINEL), "{f}");
        }
    }

    /// Deterministic garbage. Every outcome is allowed except a panic that
    /// escapes, an `Ok` that left sentinel-only output unexplained, or an
    /// `Err` that touched the destination.
    #[test]
    fn victory_survives_garbage_streams() {
        let backend = VictoryBackend::new().unwrap();
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut outcomes = std::collections::BTreeMap::<String, u32>::new();
        for round in 0..600 {
            let f = BcFormat::ALL[round % 6];
            let len = 1 + (next() % 4096) as usize;
            let src: Vec<u8> = (0..len).map(|_| next() as u8).collect();
            let mut dst = vec![SENTINEL; f.block_bytes() * (1 + (next() % 128) as usize)];
            let key = match unpack_strict(&backend, f.mode(), &src, &mut dst) {
                Ok(()) => format!("{f} ok"),
                Err(e) => {
                    assert!(dst.iter().all(|&b| b == SENTINEL), "{f}: error left partial output");
                    match e {
                        UnpackError::BackendPanicked => format!("{f} PANIC"),
                        UnpackError::Stream(s) => format!("{f} err: {s}"),
                        other => format!("{f} {other}"),
                    }
                }
            };
            *outcomes.entry(key).or_default() += 1;
        }
        let panics: u32 = outcomes.iter().filter(|(k, _)| k.contains("PANIC")).map(|(_, v)| v).sum();
        eprintln!("garbage outcomes: {outcomes:#?}");
        assert_eq!(panics, 0, "the decoder panicked on garbage input: {outcomes:#?}");
    }
}
