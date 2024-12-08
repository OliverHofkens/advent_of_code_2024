/// Returns the number of digits in a given number
fn count_digits(mut n: u64) -> u32 {
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
fn pow10(n: u32) -> u64 {
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
}
