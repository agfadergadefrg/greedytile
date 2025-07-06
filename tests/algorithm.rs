//! Validates bitset operations and cache behavior for tile constraint propagation

use greedytile::algorithm::{
    bitset::TileBitset,
    cache::{PatternKey, ViableTilesCache},
};

#[test]
fn test_bitset_operations() {
    let mut set1 = TileBitset::new(10);
    set1.insert(1);
    set1.insert(3);
    set1.insert(5);

    let mut set2 = TileBitset::new(10);
    set2.insert(3);
    set2.insert(5);
    set2.insert(7);

    let intersection = set1.intersection(&set2);
    assert_eq!(intersection.to_vec(), vec![3, 5]);
    assert!(!intersection.is_empty());
    assert_eq!(intersection.count(), 2);
}

#[test]
fn test_bitset_empty_intersection() {
    let mut set1 = TileBitset::new(10);
    set1.insert(1);
    set1.insert(2);

    let mut set2 = TileBitset::new(10);
    set2.insert(3);
    set2.insert(4);

    let intersection = set1.intersection(&set2);
    assert!(intersection.is_empty());
    assert_eq!(intersection.count(), 0);
    assert_eq!(intersection.to_vec(), vec![]);
}

#[test]
fn test_cache_behavior() {
    let mut cache = ViableTilesCache::new();

    // Verify cache returns consistent results and tracks hit/miss statistics correctly
    let key = PatternKey::new(&[[1, 2, 3], [4, 5, 6], [7, 8, 9]], 1, 1);

    let result1_vec = {
        let result1 = cache.get_or_compute_pattern(key.clone(), || {
            let mut bitset = TileBitset::new(10);
            bitset.insert(5);
            bitset
        });
        result1.to_vec()
    };

    assert_eq!(cache.stats.misses, 1);
    assert_eq!(cache.stats.hits, 0);

    let result2_vec = {
        let result2 = cache.get_or_compute_pattern(key, || {
            unreachable!("Should not compute again!");
        });
        result2.to_vec()
    };

    assert_eq!(cache.stats.hits, 1);
    assert_eq!(result1_vec, result2_vec);
}

#[test]
fn test_pattern_key_equality() {
    let pattern1 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let pattern2 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let pattern3 = [[9, 8, 7], [6, 5, 4], [3, 2, 1]];

    let key1 = PatternKey::new(&pattern1, 1, 1);
    let key2 = PatternKey::new(&pattern2, 1, 1);
    let key3 = PatternKey::new(&pattern3, 1, 1);

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}
