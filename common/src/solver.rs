/// Solves a system of two linear equations in the form:
/// a1x + b1y = k1
/// a2x + b2y = k2
pub fn solve_2x2_system(
    a1: i64,
    b1: i64,
    k1: i64,
    a2: i64,
    b2: i64,
    k2: i64,
) -> Option<(i64, i64)> {
    // Check if a1 is zero (would cause division by zero in substitution)
    if a1 == 0 {
        return None;
    }

    let numerator_y = k2 * a1 - a2 * k1;
    let denominator = b2 * a1 - a2 * b1;

    // Check if denominator is zero (parallel or coincident lines)
    // Or divisions would not be a whole number (we want integer solutions only)
    if denominator == 0 || numerator_y % denominator != 0 {
        return None;
    }

    let y = numerator_y / denominator;

    let numerator_x = k1 - b1 * y;
    if numerator_x % a1 != 0 {
        return None;
    }
    let x = numerator_x / a1;

    Some((x, y))
}

/// Unit tests for the solver
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_integer_system() {
        // Test system:
        // 2x + y = 5
        // x - y = 1
        // Solution: x = 2, y = 1
        let result = solve_2x2_system(2, 1, 5, 1, -1, 1);

        match result {
            Some((x, y)) => {
                assert_eq!(x, 2);
                assert_eq!(y, 1);
            }
            None => panic!("Expected valid solution"),
        }
    }

    #[test]
    fn test_parallel_lines() {
        // Test system:
        // 2x + y = 1
        // 2x + y = 2
        let result = solve_2x2_system(2, 1, 1, 2, 1, 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_division_by_zero() {
        // Test system with a1 = 0:
        // 0x + y = 1
        // x + y = 2
        let result = solve_2x2_system(0, 1, 1, 1, 1, 2);
        assert_eq!(result, None);
    }
}
