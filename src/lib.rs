#![warn(rust_2018_idioms, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

mod common;
mod paintable;
mod snapshot;

pub use self::{
    paintable::{Paintable, PaintableBackend},
    snapshot::SnapshotBackend,
};
