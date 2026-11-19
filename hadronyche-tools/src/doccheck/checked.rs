//! Checked arithmetic on numbers read from documents (lead ruling R21: no
//! silent saturation).
//!
//! Python's integers do not overflow, so the scripts compute exact sums and
//! products of sizes, offsets and counts. A port computes them in `i128`
//! through these functions only. On overflow the operation records
//! `INT_LIMIT` ([`super::pynum::record_int_limit`]), exactly as a number
//! beyond `i128` does when it is parsed, and returns 0. The returned value
//! is never seen: `CheckSpec::evaluate` makes the verdict `INT_LIMIT` alone.
//! A check never saturates, wraps or casts (`saturating_*`, `wrapping_*` and
//! numeric `as` are banned in check modules by a source test).

use super::pynum::record_int_limit;

fn overflow(a: i128, op: &str, b: i128) -> i128 {
    record_int_limit(&format!("{a} {op} {b}"));
    0
}

/// `a + b`, recording `INT_LIMIT` on overflow.
pub fn add(a: i128, b: i128) -> i128 {
    a.checked_add(b).unwrap_or_else(|| overflow(a, "+", b))
}

/// `a - b`, recording `INT_LIMIT` on overflow.
pub fn sub(a: i128, b: i128) -> i128 {
    a.checked_sub(b).unwrap_or_else(|| overflow(a, "-", b))
}

/// `a * b`, recording `INT_LIMIT` on overflow.
pub fn mul(a: i128, b: i128) -> i128 {
    a.checked_mul(b).unwrap_or_else(|| overflow(a, "*", b))
}

/// Python's `sum(xs)`, recording `INT_LIMIT` on overflow.
pub fn sum(xs: impl IntoIterator<Item = i128>) -> i128 {
    xs.into_iter().fold(0, add)
}

/// Python's `math.prod(xs)`, recording `INT_LIMIT` on overflow.
pub fn product(xs: impl IntoIterator<Item = i128>) -> i128 {
    xs.into_iter().fold(1, mul)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doccheck::pynum::take_int_limits;

    #[test]
    fn overflow_is_recorded_never_saturated() {
        take_int_limits();
        assert_eq!(add(2, 3), 5);
        assert_eq!(mul(-4, 5), -20);
        assert_eq!(sum([1, 2, 3]), 6);
        assert_eq!(product([2, 3, 7]), 42);
        assert!(take_int_limits().is_empty());
        let _ = mul(i128::MAX, 2);
        let _ = add(i128::MAX, 1);
        let _ = sub(i128::MIN, 1);
        let _ = sum([i128::MAX, 1]);
        let limits = take_int_limits();
        assert_eq!(limits.len(), 4, "{limits:?}");
        assert!(limits[0].contains("* 2"));
    }
}
