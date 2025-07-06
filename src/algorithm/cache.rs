use crate::algorithm::bitset::TileBitset;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Key for caching pattern compatibility results
///
/// Uniquely identifies a 3x3 tile pattern and target position
/// to avoid redundant compatibility calculations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternKey {
    pattern: Vec<i32>,
    target_row: usize,
    target_col: usize,
}

impl PatternKey {
    /// Create a pattern key from the surrounding tile pattern
    pub fn new(tile_pattern: &[[i32; 3]; 3], target_row: usize, target_col: usize) -> Self {
        let pattern = tile_pattern
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        Self {
            pattern,
            target_row,
            target_col,
        }
    }
}

impl Hash for PatternKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state);
        self.target_row.hash(state);
        self.target_col.hash(state);
    }
}

/// Memoization cache for pattern compatibility calculations
///
/// Stores previously computed viable tile sets to avoid expensive
/// pattern matching operations on repeated configurations.
#[derive(Default)]
pub struct ViableTilesCache {
    /// Pattern to viable tiles mapping
    pattern_cache: HashMap<PatternKey, TileBitset>,

    /// Cache performance statistics
    pub stats: CacheStats,
}

/// Performance metrics for cache effectiveness
#[derive(Default, Debug)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
}

impl ViableTilesCache {
    /// Create an empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve cached result or compute and store new one
    ///
    /// Uses the provided closure to compute viable tiles only when
    /// the pattern is not already cached.
    pub fn get_or_compute_pattern<F>(
        &mut self,
        pattern_key: PatternKey,
        compute_fn: F,
    ) -> &TileBitset
    where
        F: FnOnce() -> TileBitset,
    {
        use std::collections::hash_map::Entry;

        match self.pattern_cache.entry(pattern_key) {
            Entry::Occupied(entry) => {
                self.stats.hits += 1;
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                self.stats.misses += 1;
                entry.insert(compute_fn())
            }
        }
    }
}
