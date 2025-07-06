//! Wave function collapse-inspired algorithm for pattern generation using information theory and statistical analysis
//!
//! The system extracts tiles from source images, analyzes their spatial relationships,
//! and generates new patterns while maintaining statistical consistency with the source.

#![forbid(unsafe_code)]

/// Core algorithm implementation including tile selection, propagation, and deadlock resolution
pub mod algorithm;
/// Statistical analysis and pattern preprocessing for source images
pub mod analysis;
/// Input/output operations and error handling
pub mod io;
/// Mathematical utilities for interpolation and probability calculations
pub mod math;
/// Spatial grid management and tile extraction utilities
pub mod spatial;

pub use io::error::{AlgorithmError, Result};
