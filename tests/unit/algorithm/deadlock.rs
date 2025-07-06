//! Tests for deadlock resolution verifying state reversion and tile unlocking

#[cfg(test)]
mod tests {
    use greedytile::algorithm::deadlock::resolve_spatial_deadlock;
    use greedytile::algorithm::feasibility::FeasibilityCountLayer;
    use greedytile::algorithm::propagation::StepData;
    use greedytile::io::configuration::ADJACENCY_LEVELS;
    use greedytile::spatial::GridState;
    use ndarray::Array4;
    use std::collections::HashMap;

    // Complex test verifying deadlock resolution unlocks tiles, reverts probability mutations, and resets adjacency weights
    // Verified by removing tile unlocking logic during deadlock resolution
    #[test]
    fn test_deadlock_resolution_unlocks_tiles_and_reverts_state() {
        let mut grid_state = GridState::new(5, 5, 2);
        let mut feasibility_layer = FeasibilityCountLayer::new(5, 5, 2);

        if let Some(tile) = grid_state.locked_tiles.get_mut([1, 1]) {
            *tile = 2;
        }
        if let Some(tile) = grid_state.locked_tiles.get_mut([1, 2]) {
            *tile = 3;
        }
        if let Some(tile) = grid_state.locked_tiles.get_mut([2, 1]) {
            *tile = 2;
        }
        if let Some(tile) = grid_state.locked_tiles.get_mut([2, 2]) {
            *tile = 3;
        }

        for row in 0..5 {
            for col in 0..5 {
                let mut weight = 1;

                for (locked_row, locked_col) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
                    let dist =
                        ((row as i32 - locked_row).abs()).max((col as i32 - locked_col).abs());

                    for level in 1..=ADJACENCY_LEVELS {
                        if dist == level as i32 {
                            weight += (1 + ADJACENCY_LEVELS - level) as u32;
                        }
                    }
                }

                if let Some(w) = grid_state.adjacency_weights.get_mut([row, col]) {
                    *w = weight;
                }
            }
        }

        let mut selection_tally = vec![2, 2];

        let step_data = StepData {
            source_ratios: vec![0.5, 0.5],
            unique_cell_count: 2,
            grid_extension_radius: 2,
            density_correction_threshold: 0.5,
            density_correction_steepness: 10.0,
            density_minimum_strength: 0.1,
            source_tiles: vec![
                [[1, 0, 0], [0, 0, 0], [0, 0, 0]],
                [[2, 0, 0], [0, 0, 0], [0, 0, 0]],
            ],
            tile_compatibility_rules: HashMap::new(),
        };

        let mut probability_influence_matrices = Array4::<f64>::ones((2, 2, 5, 5));

        for i in 0..5 {
            for j in 0..5 {
                if let Some(val) = probability_influence_matrices.get_mut([0, 0, i, j]) {
                    *val = 1.5;
                }
                if let Some(val) = probability_influence_matrices.get_mut([0, 1, i, j]) {
                    *val = 0.5;
                }
            }
        }

        for i in 0..5 {
            for j in 0..5 {
                if let Some(val) = probability_influence_matrices.get_mut([1, 0, i, j]) {
                    *val = 0.5;
                }
                if let Some(val) = probability_influence_matrices.get_mut([1, 1, i, j]) {
                    *val = 1.5;
                }
            }
        }

        if let Some(probs) = grid_state.tile_probabilities.get_mut(0) {
            if let Some(val) = probs.get_mut([1, 1]) {
                *val *= 1.5;
            }
            if let Some(val) = probs.get_mut([1, 2]) {
                *val *= 0.5;
            }
            if let Some(val) = probs.get_mut([2, 1]) {
                *val *= 1.5;
            }
            if let Some(val) = probs.get_mut([2, 2]) {
                *val *= 0.5;
            }
        }
        if let Some(probs) = grid_state.tile_probabilities.get_mut(1) {
            if let Some(val) = probs.get_mut([1, 1]) {
                *val *= 0.5;
            }
            if let Some(val) = probs.get_mut([1, 2]) {
                *val *= 1.5;
            }
            if let Some(val) = probs.get_mut([2, 1]) {
                *val *= 0.5;
            }
            if let Some(val) = probs.get_mut([2, 2]) {
                *val *= 1.5;
            }
        }

        let contradiction_pos = [2, 2];
        let system_offset = [0, 0];

        let result = resolve_spatial_deadlock(
            &mut grid_state,
            &mut feasibility_layer,
            contradiction_pos,
            system_offset,
            &mut selection_tally,
            &step_data,
            &probability_influence_matrices,
            &mut None,
            0,
        );

        assert_eq!(
            result.tiles_unlocked, 4,
            "Should unlock 4 tiles (radius 1 due to removal count increment)"
        );
        assert_eq!(
            result.unlocked_positions.len(),
            4,
            "Should have 4 unlocked positions"
        );

        assert_eq!(
            grid_state.locked_tiles.get([1, 1]).copied(),
            Some(1),
            "Tile at [1,1] should be unlocked"
        );
        assert_eq!(
            grid_state.locked_tiles.get([1, 2]).copied(),
            Some(1),
            "Tile at [1,2] should be unlocked"
        );
        assert_eq!(
            grid_state.locked_tiles.get([2, 1]).copied(),
            Some(1),
            "Tile at [2,1] should be unlocked"
        );
        assert_eq!(
            grid_state.locked_tiles.get([2, 2]).copied(),
            Some(1),
            "Tile at [2,2] should be unlocked"
        );

        assert_eq!(
            selection_tally.first().copied(),
            Some(0),
            "Tile type 1 count should be 0 (both removed)"
        );
        assert_eq!(
            selection_tally.get(1).copied(),
            Some(0),
            "Tile type 2 count should be 0 (both removed)"
        );

        let center_weight = grid_state
            .adjacency_weights
            .get([2, 2])
            .copied()
            .unwrap_or(0);
        assert_eq!(
            center_weight, 0,
            "Center adjacency weight at [2,2] should be 0 after all tiles unlocked"
        );

        let prob_1_1_0 = grid_state
            .tile_probabilities
            .first()
            .and_then(|probs| probs.get([1, 1]))
            .copied()
            .unwrap_or(-1.0);
        let prob_1_1_1 = grid_state
            .tile_probabilities
            .get(1)
            .and_then(|probs| probs.get([1, 1]))
            .copied()
            .unwrap_or(-1.0);

        assert!(
            (prob_1_1_0 - 2.667).abs() < 0.01,
            "Probability at [1,1] for color 0 should be ~2.667 after reverting"
        );
        assert!(
            (prob_1_1_1 - 0.889).abs() < 0.01,
            "Probability at [1,1] for color 1 should be ~0.889 after reverting"
        );

        let prob_2_2_0 = grid_state
            .tile_probabilities
            .first()
            .and_then(|probs| probs.get([2, 2]))
            .copied()
            .unwrap_or(-1.0);
        let prob_2_2_1 = grid_state
            .tile_probabilities
            .get(1)
            .and_then(|probs| probs.get([2, 2]))
            .copied()
            .unwrap_or(-1.0);

        assert!(
            (prob_2_2_0 - 0.889).abs() < 0.01,
            "Probability at [2,2] for color 0 should be ~0.889 after reverting"
        );
        assert!(
            (prob_2_2_1 - 2.667).abs() < 0.01,
            "Probability at [2,2] for color 1 should be ~2.667 after reverting"
        );
    }
}
