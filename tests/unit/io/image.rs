//! Tests for PNG image export functionality including file creation and error handling

#[cfg(test)]
mod tests {

    use greedytile::io::image::export_grid_as_png;
    use greedytile::spatial::GridState;
    use std::fs;
    use std::path::Path;

    // Tests PNG file creation with alternating pattern
    // Verified by disabling file save operation
    #[test]
    fn test_export_grid_as_png_creates_file() {
        let mut grid_state = GridState::new(3, 3, 2);

        if let Some(val) = grid_state.locked_tiles.get_mut([0, 0]) {
            *val = 2;
        }
        if let Some(val) = grid_state.locked_tiles.get_mut([0, 1]) {
            *val = 3;
        }
        if let Some(val) = grid_state.locked_tiles.get_mut([1, 0]) {
            *val = 3;
        }
        if let Some(val) = grid_state.locked_tiles.get_mut([1, 1]) {
            *val = 2;
        }

        let color_mapping = vec![[255, 0, 0, 255], [0, 255, 0, 255]];

        fs::create_dir_all("data/test").ok();

        let output_path = "data/test/test_output.png";

        let result = export_grid_as_png(&grid_state, &color_mapping, output_path);

        assert!(result.is_ok(), "PNG export should succeed");

        assert!(
            Path::new(output_path).exists(),
            "PNG file should be created"
        );

        fs::remove_file(output_path).ok();
        fs::remove_dir("data/test").ok();
    }

    // Tests error when no tiles placed
    // Verified by ignoring empty grid check
    #[test]
    fn test_export_grid_as_png_empty_grid_error() {
        let grid_state = GridState::new(3, 3, 2);
        let color_mapping = vec![[255, 0, 0, 255], [0, 255, 0, 255]];

        let result = export_grid_as_png(&grid_state, &color_mapping, "data/test/empty.png");

        assert!(result.is_err(), "Should fail when no tiles are placed");
    }

    // Tests error when tile index exceeds color mapping
    // Verified by disabling bounds check
    #[test]
    fn test_export_grid_as_png_invalid_tile_index() {
        let mut grid_state = GridState::new(2, 2, 2);

        if let Some(val) = grid_state.locked_tiles.get_mut([0, 0]) {
            *val = 4;
        }

        let color_mapping = vec![[255, 0, 0, 255], [0, 255, 0, 255]];

        let result = export_grid_as_png(&grid_state, &color_mapping, "data/test/invalid.png");

        assert!(
            result.is_err(),
            "Should fail when tile index exceeds color mapping"
        );
    }
}
