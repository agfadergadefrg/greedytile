//! Cubic spline interpolation for smooth curve fitting
//!
//! Implements natural spline boundary conditions where second derivatives
//! are zero at the endpoints, providing smooth interpolation without oscillation

use std::error::Error;
use std::fmt;

/// Error type for interpolation operations
#[derive(Debug, Clone)]
pub struct InterpolationError {
    message: String,
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interpolation error: {}", self.message)
    }
}

impl Error for InterpolationError {}

impl InterpolationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Cubic spline interpolation with natural boundary conditions
///
/// Provides C2 continuous interpolation through a set of data points
/// using piecewise cubic polynomials
#[derive(Debug, Clone)]
pub struct Cubic {
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    second_derivatives: Vec<f64>,
}

impl Cubic {
    /// Create a new cubic interpolation from x and y values
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `x_values` and `y_values` have different lengths
    /// - Fewer than 2 data points are provided
    /// - Internal index access fails during spline calculation
    pub fn new(x_values: Vec<f64>, y_values: Vec<f64>) -> Result<Self, InterpolationError> {
        if x_values.len() != y_values.len() {
            return Err(InterpolationError::new(
                "x_values and y_values must have the same length",
            ));
        }

        let n = x_values.len();
        if n < 2 {
            return Err(InterpolationError::new(
                "Need at least 2 points for interpolation",
            ));
        }

        let mut second_derivatives = vec![0.0; n];

        // Natural spline: second derivative = 0 at boundaries
        let mut u = vec![0.0; n - 1];
        if let Some(sd) = second_derivatives.get_mut(0) {
            *sd = 0.0;
        }
        if let Some(u_val) = u.get_mut(0) {
            *u_val = 0.0;
        }

        for i in 1..n - 1 {
            let x_i = x_values
                .get(i)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let x_i_minus_1 = x_values
                .get(i - 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let x_i_plus_1 = x_values
                .get(i + 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let y_i = y_values
                .get(i)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let y_i_minus_1 = y_values
                .get(i - 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let y_i_plus_1 = y_values
                .get(i + 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;

            let sig = (x_i - x_i_minus_1) / (x_i_plus_1 - x_i_minus_1);
            let sd_i_minus_1 = *second_derivatives
                .get(i - 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let p = sig.mul_add(sd_i_minus_1, 2.0);

            if let Some(sd) = second_derivatives.get_mut(i) {
                *sd = (sig - 1.0) / p;
            }

            let u_i_val =
                (y_i_plus_1 - y_i) / (x_i_plus_1 - x_i) - (y_i - y_i_minus_1) / (x_i - x_i_minus_1);
            let u_i_minus_1 = *u
                .get(i - 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let u_i_final =
                sig.mul_add(-u_i_minus_1, 6.0 * u_i_val / (x_i_plus_1 - x_i_minus_1)) / p;

            if let Some(u_val) = u.get_mut(i) {
                *u_val = u_i_final;
            }
        }

        if let Some(sd) = second_derivatives.get_mut(n - 1) {
            *sd = 0.0;
        }

        for k in (0..n - 1).rev() {
            let sd_k = *second_derivatives
                .get(k)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let sd_k_plus_1 = *second_derivatives
                .get(k + 1)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            let u_k = *u
                .get(k)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;

            if let Some(sd) = second_derivatives.get_mut(k) {
                *sd = sd_k.mul_add(sd_k_plus_1, u_k);
            }
        }

        Ok(Self {
            x_values,
            y_values,
            second_derivatives,
        })
    }

    /// Evaluate the interpolation at point x
    ///
    /// Uses binary search to find the appropriate spline segment,
    /// then evaluates the cubic polynomial for that segment.
    /// Points outside the data range return the nearest boundary value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data points are available
    /// - Internal index access fails
    /// - The x values are not strictly increasing
    pub fn evaluate(&self, x: f64) -> Result<f64, InterpolationError> {
        let n = self.x_values.len();
        if n == 0 {
            return Err(InterpolationError::new("No data points available"));
        }

        let first_x = self
            .x_values
            .first()
            .ok_or_else(|| InterpolationError::new("No x values"))?;
        let first_y = self
            .y_values
            .first()
            .ok_or_else(|| InterpolationError::new("No y values"))?;

        if x <= *first_x {
            return Ok(*first_y);
        }

        let last_x = self
            .x_values
            .get(n - 1)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let last_y = self
            .y_values
            .get(n - 1)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;

        if x >= *last_x {
            return Ok(*last_y);
        }

        let mut klo = 0;
        let mut khi = n - 1;
        while khi - klo > 1 {
            let k = usize::midpoint(khi, klo);
            let x_k = self
                .x_values
                .get(k)
                .ok_or_else(|| InterpolationError::new("Invalid index"))?;
            if *x_k > x {
                khi = k;
            } else {
                klo = k;
            }
        }

        let x_khi = self
            .x_values
            .get(khi)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let x_klo = self
            .x_values
            .get(klo)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let y_khi = self
            .y_values
            .get(khi)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let y_klo = self
            .y_values
            .get(klo)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let sd_khi = self
            .second_derivatives
            .get(khi)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;
        let sd_klo = self
            .second_derivatives
            .get(klo)
            .ok_or_else(|| InterpolationError::new("Invalid index"))?;

        let h = x_khi - x_klo;
        if h <= 0.0 {
            // Binary search assumes strictly increasing x values
            return Err(InterpolationError::new(
                "x values must be strictly increasing",
            ));
        }

        let a = (x_khi - x) / h;
        let b = (x - x_klo) / h;

        Ok(a * y_klo
            + b * y_khi
            + ((a.powi(3) - a) * sd_klo + (b.powi(3) - b) * sd_khi) * h.powi(2) / 6.0)
    }
}
