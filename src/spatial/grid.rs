//! Grid state management with dynamic extension and spatial data structures
//!
//! Provides a unified interface for managing wave function collapse state including
//! probabilities, entropy, adjacency weights, and deadlock resolution counters.
//! The grid automatically extends when tile placement exceeds current bounds.

use ndarray::{Array2, Array3};
use num_traits::{NumAssign, One};

use crate::spatial::extension::{
    Extendable, calculate_extension, extend_array_2d, extend_array_3d,
};

/// Axis-aligned bounding box for generation constraints
#[derive(Debug, Clone)]
pub struct BoundingBox {
    /// Minimum coordinates (inclusive)
    pub min: [i32; 2],
    /// Maximum coordinates (inclusive)
    pub max: [i32; 2],
}

impl BoundingBox {
    /// Check if a position is within the bounds
    pub const fn contains(&self, pos: [i32; 2]) -> bool {
        pos[0] >= self.min[0]
            && pos[0] <= self.max[0]
            && pos[1] >= self.min[1]
            && pos[1] <= self.max[1]
    }
}

/// Grid state containing all wave function collapse data structures
///
/// Maintains separate 2D arrays for different state aspects to improve
/// cache locality and enable selective updates. The grid coordinates use
/// an offset system to support negative indices during extension.
#[derive(Debug, Clone)]
pub struct GridState {
    /// Probability values for each tile type (indexed by `tile_type`, `row`, `col`)
    pub tile_probabilities: Vec<Array2<f64>>,

    /// Shannon entropy calculated from tile probabilities
    pub entropy: Array2<f64>,

    /// Count of locked adjacent positions (used for propagation ordering)
    pub adjacency_weights: Array2<u32>,

    /// Locked positions with tile references (0 = unlocked, 1+ = tile index)
    pub locked_tiles: Array2<u32>,

    /// Combined score for tile selection prioritization
    pub feasibility: Array2<f64>,

    /// Deadlock recovery counter per position
    pub removal_count: Array2<u8>,

    /// Number of unique tile types
    pub unique_cell_count: usize,

    /// Current grid dimensions (rows, cols)
    pub dimensions: (usize, usize),

    /// Optional generation bounds in world coordinates
    pub generation_bounds: Option<BoundingBox>,
}

impl GridState {
    /// Create a new grid state with initial dimensions
    ///
    /// Initializes all probability matrices to 1.0 (maximum uncertainty)
    /// and other state arrays to appropriate default values.
    pub fn new(rows: usize, cols: usize, unique_cell_count: usize) -> Self {
        let dimensions = (rows, cols);

        let mut tile_probabilities = Vec::with_capacity(unique_cell_count);
        for _ in 0..unique_cell_count {
            tile_probabilities.push(Array2::ones((rows, cols)));
        }

        let entropy = Array2::ones((rows, cols));
        let adjacency_weights = Array2::ones((rows, cols));
        let locked_tiles = Array2::ones((rows, cols));
        let feasibility = Array2::ones((rows, cols));
        let removal_count = Array2::zeros((rows, cols));

        Self {
            tile_probabilities,
            entropy,
            adjacency_weights,
            locked_tiles,
            feasibility,
            removal_count,
            unique_cell_count,
            dimensions,
            generation_bounds: None,
        }
    }

    /// Get the number of rows in the grid
    pub const fn rows(&self) -> usize {
        self.dimensions.0
    }

    /// Get the number of columns in the grid
    pub const fn cols(&self) -> usize {
        self.dimensions.1
    }

    /// Extend the grid if needed to accommodate a position plus radius
    ///
    /// Returns the new offset and whether extension occurred. Extension preserves
    /// all existing data while adding padding with appropriate default values.
    /// The offset is adjusted to maintain consistent coordinate mapping.
    pub fn extend_if_needed(
        &mut self,
        offset: [i32; 2],
        coordinates: &[i32; 2],
        radius: i32,
    ) -> ([i32; 2], bool) {
        let mut extension_info =
            calculate_extension([self.rows(), self.cols()], offset, coordinates, radius);

        // Constrain extension to bounds if specified
        if let Some(bounds) = &self.generation_bounds {
            extension_info = self.constrain_extension(extension_info, bounds, offset);
        }

        if !extension_info.needs_extension {
            return (offset, false);
        }

        // Probability matrices use 1.0 padding (maximum uncertainty)
        for prob_matrix in &mut self.tile_probabilities {
            *prob_matrix = extend_array_2d(prob_matrix, &extension_info, f64::padding_value());
        }

        self.entropy = extend_array_2d(&self.entropy, &extension_info, f64::padding_value());
        self.adjacency_weights = extend_array_2d(
            &self.adjacency_weights,
            &extension_info,
            u32::padding_value(),
        );
        self.locked_tiles =
            extend_array_2d(&self.locked_tiles, &extension_info, u32::padding_value());
        self.feasibility =
            extend_array_2d(&self.feasibility, &extension_info, f64::padding_value());
        self.removal_count =
            extend_array_2d(&self.removal_count, &extension_info, u8::padding_value());

        let new_height = self.rows() + extension_info.pad_left + extension_info.pad_right;
        let new_width = self.cols() + extension_info.pad_top + extension_info.pad_bottom;
        self.dimensions = (new_height, new_width);

        (extension_info.new_offset, true)
    }

    /// Constrain extension to respect generation bounds
    const fn constrain_extension(
        &self,
        mut extension_info: crate::spatial::extension::ExtensionInfo,
        bounds: &BoundingBox,
        offset: [i32; 2],
    ) -> crate::spatial::extension::ExtensionInfo {
        // Calculate current grid bounds in world coordinates
        let current_min = [-offset[0], -offset[1]];

        // Calculate what the new bounds would be after extension
        let new_min = [
            current_min[0] - extension_info.pad_left as i32,
            current_min[1] - extension_info.pad_top as i32,
        ];

        // Constrain extension to not exceed generation bounds
        if new_min[0] < bounds.min[0] {
            let excess = (bounds.min[0] - new_min[0]) as usize;
            extension_info.pad_left = extension_info.pad_left.saturating_sub(excess);
        }

        if new_min[1] < bounds.min[1] {
            let excess = (bounds.min[1] - new_min[1]) as usize;
            extension_info.pad_top = extension_info.pad_top.saturating_sub(excess);
        }

        // Similar constraints for max bounds
        let current_max = [
            current_min[0] + self.rows() as i32 - 1,
            current_min[1] + self.cols() as i32 - 1,
        ];

        let new_max = [
            current_max[0] + extension_info.pad_right as i32,
            current_max[1] + extension_info.pad_bottom as i32,
        ];

        if new_max[0] > bounds.max[0] {
            let excess = (new_max[0] - bounds.max[0]) as usize;
            extension_info.pad_right = extension_info.pad_right.saturating_sub(excess);
        }

        if new_max[1] > bounds.max[1] {
            let excess = (new_max[1] - bounds.max[1]) as usize;
            extension_info.pad_bottom = extension_info.pad_bottom.saturating_sub(excess);
        }

        // Recalculate if extension is still needed
        extension_info.needs_extension = extension_info.pad_left
            + extension_info.pad_right
            + extension_info.pad_top
            + extension_info.pad_bottom
            > 0;

        // Update offset if extension occurs
        if extension_info.needs_extension {
            extension_info.new_offset = [
                offset[0] + extension_info.pad_left as i32,
                offset[1] + extension_info.pad_top as i32,
            ];
        } else {
            extension_info.new_offset = offset;
        }

        extension_info
    }
}

/// Get region spans for a given position and radius
///
/// Converts world coordinates to grid indices and returns ranges for
/// iteration. Automatically clamps to valid grid bounds.
pub const fn get_region_spans(
    offset: &[i32; 2],
    coordinates: &[i32; 2],
    radius: i32,
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let index = [coordinates[0] + offset[0], coordinates[1] + offset[1]];

    // Calculate bounds with proper handling of negative values
    let row_start = if index[0] - radius < 0 {
        0
    } else {
        (index[0] - radius) as usize
    };

    let col_start = if index[1] - radius < 0 {
        0
    } else {
        (index[1] - radius) as usize
    };

    // For the end bounds, we need to ensure they're non-negative before casting
    let row_end = if index[0] + radius + 1 < 0 {
        0
    } else {
        (index[0] + radius + 1) as usize
    };

    let col_end = if index[1] + radius + 1 < 0 {
        0
    } else {
        (index[1] + radius + 1) as usize
    };

    (row_start..row_end, col_start..col_end)
}

/// Generic matrix extension for 3D arrays
///
/// Used for legacy compatibility with older matrix representations.
/// Prefer `GridState::extend_if_needed` for new code.
///
/// # Panics
///
/// Panics if dimensions exceed `i32::MAX`
pub fn extend_matrices<T>(
    matrices: Array3<T>,
    offset: [i32; 2],
    coordinates: &[i32; 2],
    radius: i32,
) -> (Array3<T>, [i32; 2])
where
    T: NumAssign + One + Clone,
{
    let (_, rows, cols) = matrices.dim();
    let current_dims = [rows, cols];
    let extension_info = calculate_extension(current_dims, offset, coordinates, radius);

    if !extension_info.needs_extension {
        return (matrices, offset);
    }

    let new_matrices = extend_array_3d(&matrices, &extension_info);
    (new_matrices, extension_info.new_offset)
}
