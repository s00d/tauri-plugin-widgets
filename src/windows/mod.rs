//! Windows-only modules (Widgets Board is C#; this tree is Rust host helpers).

/// Desktop layer (`layer: "desktop"`): WorkerW / Progman parenting. Always built on
/// Windows; the `workerw` feature is kept for compatibility and no longer gates it.
pub mod workerw;

pub use workerw::{attach_to_desktop, attach_window};
