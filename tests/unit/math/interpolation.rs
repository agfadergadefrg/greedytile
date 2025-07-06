//! Tests for cubic spline interpolation and extrapolation behavior

#[cfg(test)]
mod tests {
    use crate::math::interpolation::Cubic;

    // Tests cubic spline passes through data points, interpolates smoothly, clamps extrapolation, and preserves monotonicity
    // Verified by removing cubic term to make it linear
    #[test]
    fn test_cubic_interpolation_passes_through_data_points_and_interpolates_smoothly() {
        let x_values = vec![-1.0, 0.0, 1.0, 2.0, 3.0];
        let y_values = vec![4.0, 1.0, 0.0, 1.0, 4.0];

        let cubic = Cubic::new(x_values.clone(), y_values.clone())
            .expect("Failed to create cubic interpolation");

        for (x, y) in x_values.iter().zip(y_values.iter()) {
            let interpolated = cubic
                .evaluate(*x)
                .expect("Failed to evaluate interpolation");
            assert!(
                (interpolated - y).abs() < 1e-10,
                "Interpolation should pass through data point ({x}, {y}), got {interpolated}"
            );
        }

        let test_x = 0.5;
        let interpolated = cubic
            .evaluate(test_x)
            .expect("Failed to evaluate interpolation");
        assert!(
            interpolated > 0.0 && interpolated < 0.5,
            "Interpolation at x={test_x} should be between adjacent y values, got {interpolated}"
        );

        assert!(
            (cubic
                .evaluate(-2.0)
                .expect("Failed to evaluate interpolation")
                - 4.0)
                .abs()
                < f64::EPSILON,
            "Should clamp to first y value for x < min"
        );
        assert!(
            (cubic
                .evaluate(4.0)
                .expect("Failed to evaluate interpolation")
                - 4.0)
                .abs()
                < f64::EPSILON,
            "Should clamp to last y value for x > max"
        );

        let mid_point = cubic
            .evaluate(1.5)
            .expect("Failed to evaluate interpolation");
        assert!(
            mid_point > 0.0 && mid_point < 1.0,
            "Interpolation should preserve monotonicity in [1,2]"
        );
    }
}
