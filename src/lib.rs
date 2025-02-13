// Copyright 2023 TiKV Project Authors. Licensed under Apache-2.0.

#![feature(slice_pattern)]
#![feature(let_chains)]

mod arena;
mod key;
mod list;
mod memory_control;

pub use key::{ByteWiseComparator, FixedLengthSuffixComparator, KeyComparator};
pub use list::{IterRef, Node, Skiplist};
pub use memory_control::{AllocationRecorder, MemoryLimiter, RecorderLimiter};

use tikv_jemalloc_ctl::{epoch, stats, Error};

pub type AllocStats = Vec<(&'static str, usize)>;

pub fn fetch_stats() -> Result<Option<AllocStats>, Error> {
    // Stats are cached. Need to advance epoch to refresh.
    epoch::advance()?;

    Ok(Some(vec![
        ("allocated", stats::allocated::read()?),
        ("active", stats::active::read()?),
        ("metadata", stats::metadata::read()?),
        ("resident", stats::resident::read()?),
        ("mapped", stats::mapped::read()?),
        ("retained", stats::retained::read()?),
        // (
        //     "dirty",
        //     stats::resident::read()? - stats::active::read()? - stats::metadata::read()?,
        // ),
        (
            "fragmentation",
            stats::active::read()? - stats::allocated::read()?,
        ),
    ]))
}

pub struct ReadableSize(pub u64);
const BINARY_DATA_MAGNITUDE: u64 = 1024;
pub const B: u64 = 1;
pub const KIB: u64 = B * BINARY_DATA_MAGNITUDE;
pub const MIB: u64 = KIB * BINARY_DATA_MAGNITUDE;
pub const GIB: u64 = MIB * BINARY_DATA_MAGNITUDE;
pub const TIB: u64 = GIB * BINARY_DATA_MAGNITUDE;
pub const PIB: u64 = TIB * BINARY_DATA_MAGNITUDE;

impl ReadableSize {
    pub const fn kb(count: u64) -> ReadableSize {
        ReadableSize(count * KIB)
    }

    pub const fn mb(count: u64) -> ReadableSize {
        ReadableSize(count * MIB)
    }

    pub const fn gb(count: u64) -> ReadableSize {
        ReadableSize(count * GIB)
    }

    pub const fn as_mb(self) -> u64 {
        self.0 / MIB
    }

    pub const fn as_kb(self) -> u64 {
        self.0 / KIB
    }

    pub fn as_mb_f64(self) -> f64 {
        self.0 as f64 / MIB as f64
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Bound<T> {
    /// An inclusive bound.
    Included(T),
    /// An exclusive bound.
    Excluded(T),
    /// An infinite endpoint. Indicates that there is no bound in this direction.
    Unbounded,
}
