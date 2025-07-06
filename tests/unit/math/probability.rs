//! Tests for probability distribution functions and approximations

#[cfg(test)]
mod tests {
    use crate::math::probability::binomial_normal_approximate_cdf;

    // Tests normal approximation to binomial CDF for fair coin flips with n=10, p=0.5
    // Verified by removing continuity correction
    #[test]
    fn test_binomial_normal_approximate_cdf_fair_coin_flips() {
        let n = 10;
        let p = 0.5;
        let k = 3;

        let cdf_value = binomial_normal_approximate_cdf(n, p, k);

        assert!(
            (cdf_value - 0.171_390_845_233).abs() < 1e-7,
            "Expected CDF value around 0.171390845233038, got {cdf_value}"
        );

        assert!((binomial_normal_approximate_cdf(n, p, 0) - 0.002_213_326_890_825).abs() < 1e-7);
        assert!((binomial_normal_approximate_cdf(n, p, n) - 1.0).abs() < f64::EPSILON);
        assert!((binomial_normal_approximate_cdf(n, p, n + 5) - 1.0).abs() < f64::EPSILON);
    }
}
