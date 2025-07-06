//! Tests for pattern caching behavior including hit/miss tracking and key uniqueness

#[cfg(test)]
mod tests {
    use greedytile::algorithm::bitset::TileBitset;
    use greedytile::algorithm::cache::{PatternKey, ViableTilesCache};

    // Verifies new cache starts with 0 hits and 0 misses
    // Verified by initializing cache with non-zero hit and miss counts
    #[test]
    fn test_cache_new() {
        let cache = ViableTilesCache::new();
        assert_eq!(cache.stats.hits, 0);
        assert_eq!(cache.stats.misses, 0);
    }

    // Tests pattern key creation from neighboring tiles
    // Verified by making pattern key equality always return false
    #[test]
    fn test_pattern_key_creation() {
        let pattern = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let key = PatternKey::new(&pattern, 1, 1);

        let pattern2 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let key2 = PatternKey::new(&pattern2, 1, 1);

        assert_eq!(key, key2);
    }

    // Tests cache miss on first access and hit on second
    // Verified by removing hit counter increment logic
    #[test]
    fn test_cache_miss_and_hit() {
        let mut cache = ViableTilesCache::new();
        let pattern = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let key = PatternKey::new(&pattern, 1, 1);

        let mut compute_count = 0;
        let result1_vec = {
            let result1 = cache.get_or_compute_pattern(key.clone(), || {
                compute_count += 1;
                let mut bitset = TileBitset::new(10);
                bitset.insert(5);
                bitset
            });
            result1.to_vec()
        };

        assert_eq!(cache.stats.misses, 1);
        assert_eq!(cache.stats.hits, 0);
        assert_eq!(result1_vec, vec![5]);
        assert_eq!(compute_count, 1);

        let result2_vec = {
            let result2 = cache.get_or_compute_pattern(key, || {
                compute_count += 1;
                let mut bitset = TileBitset::new(10);
                bitset.insert(99);
                bitset
            });
            result2.to_vec()
        };

        assert_eq!(cache.stats.hits, 1);
        assert_eq!(cache.stats.misses, 1);
        assert_eq!(result2_vec, vec![5]);
        assert_eq!(compute_count, 1);
    }

    // Tests different patterns produce different cache entries
    // Verified by making pattern key ignore actual pattern data
    #[test]
    fn test_different_patterns_different_results() {
        let mut cache = ViableTilesCache::new();

        let pattern1 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let key1 = PatternKey::new(&pattern1, 1, 1);

        let pattern2 = [[9, 8, 7], [6, 5, 4], [3, 2, 1]];
        let key2 = PatternKey::new(&pattern2, 1, 1);

        let result1_vec = {
            let result1 = cache.get_or_compute_pattern(key1, || {
                let mut bitset = TileBitset::new(10);
                bitset.insert(1);
                bitset
            });
            result1.to_vec()
        };

        let result2_vec = {
            let result2 = cache.get_or_compute_pattern(key2, || {
                let mut bitset = TileBitset::new(10);
                bitset.insert(2);
                bitset
            });
            result2.to_vec()
        };

        assert_eq!(result1_vec, vec![1]);
        assert_eq!(result2_vec, vec![2]);
        assert_eq!(cache.stats.misses, 2);
        assert_eq!(cache.stats.hits, 0);
    }
}
