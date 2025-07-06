//! Dynamic grid extension utilities for adaptive spatial data structures
//!
//! Provides efficient extension of n-dimensional arrays when tile placement
//! exceeds current bounds. The extension system calculates minimal padding
//! requirements and preserves existing data while avoiding repeated allocations.

use ndarray::{Array2, Array3};
use num_traits::{NumAssign, One};

/// Trait for types that can be extended with padding
pub trait Extendable {
    /// The value to use for padding new cells
    fn padding_value() -> Self;
}

/// Extension information calculated from current bounds and target position
///
/// Stores padding requirements for each direction and the updated coordinate
/// system offset. Minimizes memory allocation by calculating exact requirements.
#[derive(Debug, Clone, Copy)]
pub struct ExtensionInfo {
    /// Padding needed on the left side
    pub pad_left: usize,
    /// Padding needed on the right side
    pub pad_right: usize,
    /// Padding needed on the top
    pub pad_top: usize,
    /// Padding needed on the bottom
    pub pad_bottom: usize,
    /// Updated coordinate system offset after extension
    pub new_offset: [i32; 2],
    /// Whether extension is actually required
    pub needs_extension: bool,
}

/// Calculate extension information for a grid
///
/// Returns a struct containing padding requirements and new offset values
/// when extending a grid to accommodate a new position plus radius.
/// Preserves the coordinate system by adjusting the offset appropriately.
pub fn calculate_extension(
    current_dims: [usize; 2],
    offset: [i32; 2],
    coordinates: &[i32; 2],
    radius: i32,
) -> ExtensionInfo {
    let current_dims_i32 = [current_dims[0] as i32, current_dims[1] as i32];
    let current_min = [-offset[0], -offset[1]];
    let current_max = [
        -offset[0] + current_dims_i32[0] - 1,
        -offset[1] + current_dims_i32[1] - 1,
    ];

    let new_min = [
        current_min[0].min(coordinates[0] - radius),
        current_min[1].min(coordinates[1] - radius),
    ];
    let new_max = [
        current_max[0].max(coordinates[0] + radius),
        current_max[1].max(coordinates[1] + radius),
    ];

    let pad_left = (current_min[0] - new_min[0]) as usize;
    let pad_right = (new_max[0] - current_max[0]) as usize;
    let pad_top = (current_min[1] - new_min[1]) as usize;
    let pad_bottom = (new_max[1] - current_max[1]) as usize;

    let needs_extension = pad_left + pad_right + pad_top + pad_bottom > 0;

    let new_offset = if needs_extension {
        [offset[0] + pad_left as i32, offset[1] + pad_top as i32]
    } else {
        offset
    };

    ExtensionInfo {
        pad_left,
        pad_right,
        pad_top,
        pad_bottom,
        new_offset,
        needs_extension,
    }
}

/// Extend a 2D array with padding
///
/// Copies existing data to the appropriate position in the new array
/// while filling new cells with the specified padding value. Returns
/// the original array unchanged if no extension is needed.
pub fn extend_array_2d<T: Clone>(
    array: &Array2<T>,
    info: &ExtensionInfo,
    padding_value: T,
) -> Array2<T> {
    if !info.needs_extension {
        return array.clone();
    }

    let (old_rows, old_cols) = array.dim();
    let new_shape = [
        old_rows + info.pad_left + info.pad_right,
        old_cols + info.pad_top + info.pad_bottom,
    ];

    let mut new_array = Array2::from_elem(new_shape, padding_value);

    // O(mn) copy preserves spatial relationships
    for i in 0..old_rows {
        for j in 0..old_cols {
            if let (Some(src), Some(dst)) = (
                array.get([i, j]),
                new_array.get_mut([i + info.pad_left, j + info.pad_top]),
            ) {
                *dst = src.clone();
            }
        }
    }

    new_array
}

/// Extend a 3D array with padding
///
/// Maintains the layer structure while extending spatial dimensions.
/// Used for probability matrices where each layer represents a tile type.
pub fn extend_array_3d<T>(array: &Array3<T>, info: &ExtensionInfo) -> Array3<T>
where
    T: NumAssign + One + Clone,
{
    if !info.needs_extension {
        return array.clone();
    }

    let (n_layers, old_rows, old_cols) = array.dim();
    let new_shape = [
        n_layers,
        old_rows + info.pad_left + info.pad_right,
        old_cols + info.pad_top + info.pad_bottom,
    ];

    let mut new_array = Array3::<T>::ones(new_shape);

    // O(mn) copy preserves spatial relationships
    for i in 0..n_layers {
        for j in 0..old_rows {
            for k in 0..old_cols {
                if let (Some(src), Some(dst)) = (
                    array.get([i, j, k]),
                    new_array.get_mut([i, j + info.pad_left, k + info.pad_top]),
                ) {
                    *dst = src.clone();
                }
            }
        }
    }

    new_array
}

impl Extendable for f64 {
    fn padding_value() -> Self {
        1.0
    }
}

impl Extendable for u32 {
    fn padding_value() -> Self {
        1
    }
}

impl Extendable for u8 {
    fn padding_value() -> Self {
        1
    }
}
