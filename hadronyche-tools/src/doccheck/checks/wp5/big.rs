//! Python's unbounded `int`, for the numbers the WP5 scripts read with
//! `int()`: an offset or a width of any length compares, adds and multiplies
//! exactly, as it did in the script (a Rust `i64` would overflow or refuse a
//! twenty-digit cell that Python reads).

use std::cmp::Ordering;
use std::fmt;

const BASE: u64 = 1_000_000_000;

/// A signed integer of any size (base 10⁹ limbs, least significant first).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct Big {
    neg: bool,
    mag: Vec<u32>,
}

impl Big {
    /// From ASCII decimal digits (no sign); `None` when empty or not digits.
    pub fn from_digits(digits: &str) -> Option<Big> {
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let bytes = digits.as_bytes();
        let mut mag = Vec::new();
        let mut end = bytes.len();
        while end > 0 {
            let start = end.saturating_sub(9);
            let chunk = std::str::from_utf8(&bytes[start..end]).ok()?;
            mag.push(chunk.parse::<u32>().ok()?);
            end = start;
        }
        Some(Big { neg: false, mag }.trimmed())
    }

    /// From digit values in `base` (2 to 36), most significant first.
    pub fn from_radix(values: &[u32], base: u32) -> Big {
        let mut out = Big::default();
        for &v in values {
            out = out.mul(&Big::from(i64::from(base))).add(&Big::from(i64::from(v)));
        }
        out
    }

    fn trimmed(mut self) -> Big {
        while self.mag.last() == Some(&0) {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.neg = false;
        }
        self
    }

    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    pub fn neg(&self) -> Big {
        Big { neg: !self.neg, mag: self.mag.clone() }.trimmed()
    }

    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for k in 0..a.len().max(b.len()) {
            let s = u64::from(*a.get(k).unwrap_or(&0)) + u64::from(*b.get(k).unwrap_or(&0)) + carry;
            out.push((s % BASE) as u32);
            carry = s / BASE;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }

    /// `a - b` for `|a| >= |b|`.
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (k, &ak) in a.iter().enumerate() {
            let mut d = i64::from(ak) - i64::from(*b.get(k).unwrap_or(&0)) - borrow;
            borrow = 0;
            if d < 0 {
                d += BASE as i64;
                borrow = 1;
            }
            out.push(d as u32);
        }
        out
    }

    pub fn add(&self, o: &Big) -> Big {
        if self.neg == o.neg {
            return Big { neg: self.neg, mag: Big::add_mag(&self.mag, &o.mag) }.trimmed();
        }
        match Big::cmp_mag(&self.mag, &o.mag) {
            Ordering::Less => Big { neg: o.neg, mag: Big::sub_mag(&o.mag, &self.mag) }.trimmed(),
            _ => Big { neg: self.neg, mag: Big::sub_mag(&self.mag, &o.mag) }.trimmed(),
        }
    }

    pub fn sub(&self, o: &Big) -> Big {
        self.add(&o.neg())
    }

    pub fn mul(&self, o: &Big) -> Big {
        let mut acc = vec![0u64; self.mag.len() + o.mag.len() + 1];
        for (i, &a) in self.mag.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &b) in o.mag.iter().enumerate() {
                let t = acc[i + j] + u64::from(a) * u64::from(b) + carry;
                acc[i + j] = t % BASE;
                carry = t / BASE;
            }
            let mut k = i + o.mag.len();
            while carry > 0 {
                let t = acc[k] + carry;
                acc[k] = t % BASE;
                carry = t / BASE;
                k += 1;
            }
        }
        Big { neg: self.neg != o.neg, mag: acc.into_iter().map(|d| d as u32).collect() }.trimmed()
    }

    /// Python `divmod(self, d)` for a positive `d`: floor division.
    pub fn divmod_small(&self, d: u32) -> (Big, u32) {
        let mut q = vec![0u32; self.mag.len()];
        let mut rem = 0u64;
        for k in (0..self.mag.len()).rev() {
            let cur = rem * BASE + u64::from(self.mag[k]);
            q[k] = (cur / u64::from(d)) as u32;
            rem = cur % u64::from(d);
        }
        let q = Big { neg: self.neg, mag: q }.trimmed();
        if self.neg && rem != 0 {
            (q.sub(&Big::from(1)), d - rem as u32)
        } else {
            (q, rem as u32)
        }
    }

    /// Python `int.bit_length()` of the magnitude.
    pub fn bit_length(&self) -> u64 {
        let mut n = Big { neg: false, mag: self.mag.clone() };
        let mut bits = 0;
        while !n.is_zero() {
            n = n.divmod_small(2).0;
            bits += 1;
        }
        bits
    }

    /// The value when it fits an `i64`.
    pub fn to_i64(&self) -> Option<i64> {
        if self.mag.len() > 3 {
            return None;
        }
        let mut v: i128 = 0;
        for &d in self.mag.iter().rev() {
            v = v * BASE as i128 + i128::from(d);
        }
        if self.neg {
            v = -v;
        }
        i64::try_from(v).ok()
    }
}

impl From<i64> for Big {
    fn from(v: i64) -> Big {
        let mut m = v.unsigned_abs();
        let mut mag = Vec::new();
        while m > 0 {
            mag.push((m % BASE) as u32);
            m /= BASE;
        }
        Big { neg: v < 0, mag }.trimmed()
    }
}

impl Ord for Big {
    fn cmp(&self, o: &Big) -> Ordering {
        match (self.neg, o.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Big::cmp_mag(&self.mag, &o.mag),
            (true, true) => Big::cmp_mag(&o.mag, &self.mag),
        }
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, o: &Big) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl fmt::Display for Big {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mag.is_empty() {
            return write!(f, "0");
        }
        if self.neg {
            write!(f, "-")?;
        }
        write!(f, "{}", self.mag[self.mag.len() - 1])?;
        for d in self.mag.iter().rev().skip(1) {
            write!(f, "{d:09}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Big {
        match s.strip_prefix('-') {
            Some(d) => Big::from_digits(d).unwrap().neg(),
            None => Big::from_digits(s).unwrap(),
        }
    }

    #[test]
    fn arithmetic_matches_python() {
        let x = b("123456789012345678901234567890");
        assert_eq!(x.mul(&Big::from(8)).to_string(), "987654312098765431209876543120");
        assert_eq!(x.add(&b("-123456789012345678901234567891")).to_string(), "-1");
        assert_eq!(b("7").sub(&b("10")).to_string(), "-3");
        assert_eq!(b("-7").divmod_small(8), (b("-1"), 1));
        assert_eq!(b("17").divmod_small(8), (b("2"), 1));
        assert_eq!(b("0000").to_string(), "0");
        assert!(b("-5") < b("3") && b("1000000000000") > b("999999999999"));
        assert_eq!(b("9223372036854775807").to_i64(), Some(i64::MAX));
        assert_eq!(b("9223372036854775808").to_i64(), None);
        assert_eq!(Big::from_radix(&[1, 15], 16), Big::from(31));
        assert_eq!((Big::from(255).bit_length(), Big::from(256).bit_length(), Big::default().bit_length()), (8, 9, 0));
    }
}
