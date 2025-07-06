//! Algorithm constants and runtime configuration defaults

// Algorithm-specific constants for position and tile selection
/// Number of top adjacency candidates to consider
pub const ADJACENCY_CANDIDATES_CONSIDERED: usize = 30;
/// Number of top candidates to consider
pub const CANDIDATES_CONSIDERED: usize = 15;

/// Size of tile patterns (must be odd for center-based operations)
pub const TILE_SIZE: usize = 3;

/// Maximum distance for pattern influence effects
pub const PATTERN_INFLUENCE_DISTANCE: usize = 6;

/// Radius for grid extension operations
pub const GRID_EXTENSION_RADIUS: usize = 6;

// Safety limit to prevent excessive memory allocation
/// Maximum allowed grid dimension
pub const MAX_GRID_DIMENSION: usize = 10_000;

/// Initial radius for deadlock resolution
pub const BASE_REMOVAL_RADIUS: i32 = 0;

// Prevents deadlock resolution from clearing entire grid
/// Maximum radius for deadlock resolution
pub const MAX_REMOVAL_RADIUS: i32 = 6;

// Determines influence distance for pattern matching
/// Number of adjacency levels to check
pub const ADJACENCY_LEVELS: usize = 2;

// Progress bar display settings
/// Threshold for switching to batch progress mode
pub const MAX_INDIVIDUAL_PROGRESS_BARS: usize = 5;
/// Width of progress bars in characters
pub const PROGRESS_BAR_WIDTH: u16 = 50;

// Default values for configurable parameters
/// Fixed seed for reproducible generation
pub const DEFAULT_SEED: u64 = 42;

/// Default maximum iterations before stopping
pub const DEFAULT_MAX_ITERATIONS: usize = 1000;

// Output settings
/// Suffix added to output filenames
pub const OUTPUT_SUFFIX: &str = "_result";
/// Delay between GIF animation frames
pub const GIF_FRAME_DELAY_MS: u32 = 5;
/// Minimum frame delay that viewers reliably support (in milliseconds)
pub const VIEWER_MIN_FRAME_DELAY_MS: u32 = 50;
