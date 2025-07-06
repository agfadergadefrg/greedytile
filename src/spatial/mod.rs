//! Spatial data structures and grid manipulation
//!
//! This module contains spatial-related functionality including:
//! - Grid manipulation and extension
//! - Grid state management
//! - Tile data structures and extraction

/// Grid extension utilities
pub mod extension;
/// Grid state management and manipulation functions
pub mod grid;
/// Tile extraction and pattern matching utilities
pub mod tiles;

pub use grid::GridState;
