#![warn(rust_2018_idioms, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

mod snapshot;

pub use self::snapshot::SnapshotBackend;
