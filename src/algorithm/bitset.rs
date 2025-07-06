use bitvec::prelude::*;
use std::fmt;

/// Fixed-size bitset for tracking tile membership in compatibility sets
///
/// Uses 1-based indexing to match tile references throughout the system.
/// Provides O(1) membership testing and efficient set operations.
#[derive(Clone, Debug)]
pub struct TileBitset {
    bits: BitVec,
    max_tiles: usize,
}

impl TileBitset {
    /// Create a bitset with no tiles present
    pub fn new(max_tiles: usize) -> Self {
        Self {
            bits: bitvec![0; max_tiles],
            max_tiles,
        }
    }

    /// Create a bitset containing all possible tiles
    pub fn all(max_tiles: usize) -> Self {
        Self {
            bits: bitvec![1; max_tiles],
            max_tiles,
        }
    }

    /// Insert a tile index
    ///
    /// Takes 1-based tile indices, storing at index-1 internally
    pub fn insert(&mut self, tile: usize) {
        if tile > 0 && tile <= self.max_tiles {
            self.bits.set(tile - 1, true);
        }
    }

    /// Test tile membership
    pub fn contains(&self, tile: usize) -> bool {
        if tile > 0 {
            self.bits.get(tile - 1).as_deref() == Some(&true)
        } else {
            false
        }
    }

    /// Intersect this bitset with another in-place
    pub fn intersect_with(&mut self, other: &Self) {
        self.bits &= &other.bits;
    }

    /// Create a new bitset containing the intersection
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.intersect_with(other);
        result
    }

    /// Test if no tiles are present
    pub fn is_empty(&self) -> bool {
        self.bits.not_any()
    }

    /// Count tiles in the set
    pub fn count(&self) -> usize {
        self.bits.count_ones()
    }

    /// Extract all tile indices as a vector
    ///
    /// Returns 1-based indices matching the tile reference system
    pub fn to_vec(&self) -> Vec<usize> {
        self.bits.iter_ones().map(|index| index + 1).collect()
    }

    /// Convert from `HashSet` representation
    pub fn from_hashset(set: &std::collections::HashSet<usize>, max_tiles: usize) -> Self {
        let mut bitset = Self::new(max_tiles);
        for &tile in set {
            bitset.insert(tile);
        }
        bitset
    }
}

impl fmt::Display for TileBitset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TileBitset({} tiles: {:?})", self.count(), self.to_vec())
    }
}
