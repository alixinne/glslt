//! API wrapper module

#[cfg(any(feature = "cli", feature = "python"))]
mod common;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "python")]
pub mod python;
