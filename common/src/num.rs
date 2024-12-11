/// Returns the number of digits in a given number
pub fn count_digits(mut n: u64) -> u32 {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

/// Computes 10^n
pub fn pow10(n: u32) -> u64 {
    let mut result = 1;
    let mut i = 0;
    while i < n {
        result *= 10;
        i += 1;
    }
    result
}

/// Concatenates two numbers efficiently without using string operations
#[inline]
pub fn concat(a: u64, b: u64) -> Option<u64> {
    let b_digits = count_digits(b);
    let base = a.checked_mul(pow10(b_digits))?;
    base.checked_add(b)
}

/// Splits a number in 2 at index 'at'.
pub fn split(num: u64, at: u32) -> (u64, u64) {
    let factor = pow10(at);
    let first_part = num / factor;
    let last_part = num - first_part * factor;
    (first_part, last_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_counting() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(1), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(100), 3);
    }

    #[test]
    fn test_concat() {
        assert_eq!(concat(123, 456), Some(123456));
        assert_eq!(concat(1, 2), Some(12));
        assert_eq!(concat(0, 123), Some(123));
        assert_eq!(concat(123, 0), Some(1230));
    }

    #[test]
    fn test_split() {
        assert_eq!(split(123456, 3), (123, 456));
        assert_eq!(split(4444, 2), (44, 44));
    }
}
