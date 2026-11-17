//! Python numbers as the scripts used them: `int()` beyond `i128` (red by
//! rule, `INT_LIMIT`), `float()`, float formatting (`repr`, `str`,
//! `format`/f-string specs, `%`), `eval` over `+ - * **`, and
//! `fractions.Fraction`.
//!
//! Every behaviour here is proven against Python by
//! `data/pycompat_corpus.txt` (`tools/gen_pycompat_corpus.py`, run with
//! Python 3.10 and 3.13, which agree), without running Python.
//!
//! **Integers beyond `i128` are red by rule (`INT_LIMIT`).** Python's `int`
//! is unbounded; the library's is `i128`. A value beyond it (from
//! [`super::pycompat::parse_int`], a TOML big integer read with
//! [`super::pyvalue::PyValue::py_int`], an [`eval_arith`] result, an exact
//! [`Fraction`] of a tiny float) is recorded on the thread, and
//! [`crate::doccheck::checks::CheckSpec::evaluate`] turns the verdict red
//! under `INT_LIMIT` alone, like `REGEX_LIMIT` (R19): the port never
//! compares a wrapped or saturated number. The documents hold no integer
//! near 2^127; a check that meets one is stricter than its script, which the
//! harness classifies `IntLimit`.

use std::cell::RefCell;

use super::pycompat::{IntError, PyStr, parse_int};
use super::pyvalue::repr_float;

/// The criterion of an integer beyond `i128`.
pub const INT_LIMIT: &str = "INT_LIMIT";

thread_local! {
    static INT_LIMITS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Records that `what` (the text of the number or expression) is beyond
/// `i128`. Called by the library wherever such a value appears.
pub fn record_int_limit(what: &str) {
    INT_LIMITS.with(|f| f.borrow_mut().push(what.chars().take(200).collect()));
}

/// The integers beyond `i128` recorded on this thread, emptied (for
/// `CheckSpec::evaluate` only; a source test forbids it in checks).
pub fn take_int_limits() -> Vec<String> {
    INT_LIMITS.with(|f| std::mem::take(&mut *f.borrow_mut()))
}

/// Puts back the records an enclosing evaluation took.
pub fn restore_int_limits(outer: Vec<String>) {
    INT_LIMITS.with(|f| {
        let mut f = f.borrow_mut();
        let mine = std::mem::take(&mut *f);
        *f = outer;
        f.extend(mine);
    });
}

/// `float(s)`: surrounding white space, a sign, decimal digits (any Unicode
/// decimal digit), single `_` between digits, a fraction, an exponent,
/// `inf`/`infinity`/`nan` in any case. `None` is Python's `ValueError`.
pub fn py_float(s: &str) -> Option<f64> {
    let t = s.py_strip();
    let mut ascii = String::with_capacity(t.len());
    let chars: Vec<char> = t.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' {
            let digit = |k: Option<&char>| k.is_some_and(|d| digit_value(*d).is_some());
            if !(i > 0 && digit(chars.get(i - 1)) && digit(chars.get(i + 1))) {
                return None;
            }
            continue;
        }
        match digit_value(c) {
            Some(d) => ascii.push(char::from(b'0' + d)),
            None if c.is_ascii() => ascii.push(c),
            None => return None,
        }
    }
    let body = ascii.trim_start_matches(['+', '-']);
    let word = body.to_ascii_lowercase();
    if matches!(word.as_str(), "inf" | "infinity" | "nan") {
        return ascii.parse::<f64>().ok();
    }
    let valid = body
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'))
        && body.chars().any(|c| c.is_ascii_digit())
        && ascii.len() - body.len() <= 1;
    if !valid {
        return None;
    }
    ascii.parse::<f64>().ok()
}

fn digit_value(c: char) -> Option<u8> {
    let s = c.to_string();
    if c.is_ascii_digit() {
        return Some(c as u8 - b'0');
    }
    parse_int(&s).ok().and_then(|v| u8::try_from(v).ok())
}

/// `str(x)` (= `repr(x)` since Python 3).
pub fn str_float(x: f64) -> String {
    repr_float(x)
}

/// `format(x, spec)` / `f"{x:spec}"` for a float, with `spec` one of
/// `[+][.P](f|e|g)`, `""` (= `str`), `"repr"`, `"str"`. `None` for any
/// other spec (not used by a script).
pub fn format_float(x: f64, spec: &str) -> Option<String> {
    if spec.is_empty() || spec == "repr" || spec == "str" {
        return Some(repr_float(x));
    }
    let (plus, rest) = match spec.strip_prefix('+') {
        Some(r) => (true, r),
        None => (false, spec),
    };
    let kind = rest.chars().last()?;
    let prec = match rest[..rest.len() - 1].strip_prefix('.') {
        Some(p) => Some(usize::try_from(parse_int(p).ok()?).ok()?),
        None if rest.len() == 1 => None,
        None => return None,
    };
    let body = match kind {
        'f' => fixed(x, prec.unwrap_or(6)),
        'e' => sci(x, prec.unwrap_or(6)),
        'g' => general(x, prec.unwrap_or(6)),
        _ => return None,
    };
    Some(if plus && !body.starts_with('-') {
        format!("+{body}")
    } else {
        body
    })
}

/// `("%" + spec) % x` for a float: `.P(f|e|g)`, `g`, `d` (truncated toward
/// zero), `s` (`str`). `None` for any other spec.
pub fn percent_float(x: f64, spec: &str) -> Option<String> {
    match spec {
        "s" => Some(repr_float(x)),
        "d" => {
            if !x.is_finite() {
                return None;
            }
            let t = x.trunc();
            Some(if t == 0.0 {
                "0".into()
            } else {
                format!("{t:.0}")
            })
        }
        _ => format_float(x, spec),
    }
}

fn special(x: f64) -> Option<String> {
    if x.is_nan() {
        Some("nan".into())
    } else if x.is_infinite() {
        Some(if x > 0.0 { "inf".into() } else { "-inf".into() })
    } else {
        None
    }
}

fn fixed(x: f64, prec: usize) -> String {
    special(x).unwrap_or_else(|| format!("{x:.prec$}"))
}

/// Rust's `{:.Pe}` (`1.235e3`) as Python writes it (`1.235e+03`).
fn sci(x: f64, prec: usize) -> String {
    if let Some(s) = special(x) {
        return s;
    }
    let r = format!("{x:.prec$e}");
    let (m, e) = r.split_once('e').unwrap_or((&r, "0"));
    let exp = parse_int(e).unwrap_or_default();
    format!("{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
}

/// Python's `g`: `P` significant digits (0 counts as 1); exponent form
/// when the exponent of the rounded value is below -4 or at least `P`;
/// trailing zeros (and a trailing dot) removed.
fn general(x: f64, prec: usize) -> String {
    if let Some(s) = special(x) {
        return s;
    }
    let p = prec.max(1);
    let r = format!("{x:.*e}", p - 1);
    let (_, e) = r.split_once('e').unwrap_or((&r, "0"));
    let exp = parse_int(e).unwrap_or_default();
    let p_i = i128::try_from(p).unwrap_or(i128::MAX);
    if exp < -4 || exp >= p_i {
        let s = sci(x, p - 1);
        let (m, e) = s.split_once('e').unwrap_or((&s, ""));
        format!("{}e{e}", strip_zeros(m))
    } else {
        let decimals = usize::try_from(p_i - 1 - exp).unwrap_or_default();
        strip_zeros(&format!("{x:.decimals$}"))
    }
}

fn strip_zeros(m: &str) -> String {
    if m.contains('.') {
        m.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        m.to_string()
    }
}

/// A Python number: what `eval` of an arithmetic expression returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyNum {
    Int(i128),
    Float(f64),
}

impl PyNum {
    /// `repr` of the number.
    pub fn repr(self) -> String {
        match self {
            PyNum::Int(n) => n.to_string(),
            PyNum::Float(f) => repr_float(f),
        }
    }

    /// The number as a float (`float(n)`).
    pub fn as_f64(self) -> f64 {
        match self {
            // exact below 2^53; beyond, the nearest float, as Python
            #[allow(clippy::cast_precision_loss)]
            PyNum::Int(n) => n as f64,
            PyNum::Float(f) => f,
        }
    }

    /// Python's `==` against an integer.
    pub fn eq_int(self, n: i128) -> bool {
        match self {
            PyNum::Int(m) => m == n,
            #[allow(clippy::cast_precision_loss)]
            PyNum::Float(f) => f == n as f64 && f.fract() == 0.0,
        }
    }
}

/// Why [`eval_arith`] has no value: the exception the script's `eval`
/// raised, or an integer beyond `i128` (recorded: `INT_LIMIT`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    SyntaxError,
    ZeroDivisionError,
    OverflowError,
    IntLimit,
}

impl EvalError {
    /// The Python exception class (`IntLimit` has none: Python carries on).
    pub fn name(&self) -> &'static str {
        match self {
            EvalError::SyntaxError => "SyntaxError",
            EvalError::ZeroDivisionError => "ZeroDivisionError",
            EvalError::OverflowError => "OverflowError",
            EvalError::IntLimit => "INT_LIMIT",
        }
    }
}

/// `eval(expr)` for an expression of decimal literals (`12`, `1.5`), `+`,
/// `-`, `*`, `**`, unary `+`/`-` and spaces, with Python's precedence
/// (`-2**2` is -4, `2**3**2` is 512), its literal rule (`01` is a syntax
/// error, `00` is 0) and its types (`2**-1` is the float 0.5).
pub fn eval_arith(expr: &str) -> Result<PyNum, EvalError> {
    let toks = tokens(expr)?;
    let mut p = Parser { toks: &toks, at: 0 };
    let v = p.sum()?;
    if p.at != toks.len() {
        return Err(EvalError::SyntaxError);
    }
    Ok(v)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(PyNum),
    Plus,
    Minus,
    Star,
    Pow,
}

fn tokens(s: &str) -> Result<Vec<Tok>, EvalError> {
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            ' ' | '\t' => {}
            '+' => out.push(Tok::Plus),
            '-' => out.push(Tok::Minus),
            '*' if c.get(i + 1) == Some(&'*') => {
                out.push(Tok::Pow);
                i += 1;
            }
            '*' => out.push(Tok::Star),
            d if d.is_ascii_digit() || d == '.' => {
                let start = i;
                while i + 1 < c.len() && (c[i + 1].is_ascii_digit() || c[i + 1] == '.') {
                    i += 1;
                }
                let lit: String = c[start..=i].iter().collect();
                out.push(Tok::Num(literal(&lit)?));
            }
            _ => return Err(EvalError::SyntaxError),
        }
        i += 1;
    }
    Ok(out)
}

fn literal(lit: &str) -> Result<PyNum, EvalError> {
    if lit.contains('.') {
        if lit.matches('.').count() > 1 || lit == "." {
            return Err(EvalError::SyntaxError);
        }
        return py_float(lit)
            .map(PyNum::Float)
            .ok_or(EvalError::SyntaxError);
    }
    if lit.len() > 1 && lit.starts_with('0') && lit.chars().any(|x| x != '0') {
        return Err(EvalError::SyntaxError);
    }
    match parse_int(lit) {
        Ok(n) => Ok(PyNum::Int(n)),
        Err(IntError::Overflow(_)) => Err(EvalError::IntLimit),
        Err(IntError::NotInt(_)) => Err(EvalError::SyntaxError),
    }
}

struct Parser<'a> {
    toks: &'a [Tok],
    at: usize,
}

type Val = Result<PyNum, EvalError>;

fn int_limit(what: &str) -> EvalError {
    record_int_limit(what);
    EvalError::IntLimit
}

fn arith(a: PyNum, b: PyNum, op: &Tok) -> Val {
    use PyNum::{Float, Int};
    match (a, b) {
        (Int(x), Int(y)) => match op {
            Tok::Plus => x.checked_add(y),
            Tok::Minus => x.checked_sub(y),
            _ => x.checked_mul(y),
        }
        .map(Int)
        .ok_or_else(|| int_limit(&format!("{x} {op:?} {y}"))),
        _ => {
            let (x, y) = (a.as_f64(), b.as_f64());
            Ok(Float(match op {
                Tok::Plus => x + y,
                Tok::Minus => x - y,
                _ => x * y,
            }))
        }
    }
}

fn power(base: PyNum, exp: PyNum) -> Val {
    use PyNum::{Float, Int};
    match (base, exp) {
        (Int(b), Int(e)) if e >= 0 => u32::try_from(e)
            .ok()
            .and_then(|e| b.checked_pow(e))
            .map(Int)
            .ok_or_else(|| int_limit(&format!("{b}**{e}"))),
        (Int(0), Int(_)) => Err(EvalError::ZeroDivisionError),
        _ => {
            let (b, e) = (base.as_f64(), exp.as_f64());
            if b == 0.0 && e < 0.0 {
                return Err(EvalError::ZeroDivisionError);
            }
            let v = b.powf(e);
            if v.is_infinite() {
                return Err(EvalError::OverflowError);
            }
            Ok(Float(v))
        }
    }
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn sum(&mut self) -> Val {
        let mut v = self.term()?;
        while let Some(t) = self.peek().cloned() {
            if t != Tok::Plus && t != Tok::Minus {
                break;
            }
            self.at += 1;
            let r = self.term()?;
            v = arith(v, r, &t)?;
        }
        Ok(v)
    }

    fn term(&mut self) -> Val {
        let mut v = self.unary()?;
        while self.peek() == Some(&Tok::Star) {
            self.at += 1;
            let r = self.unary()?;
            v = arith(v, r, &Tok::Star)?;
        }
        Ok(v)
    }

    fn unary(&mut self) -> Val {
        match self.peek() {
            Some(Tok::Plus) => {
                self.at += 1;
                self.unary()
            }
            Some(Tok::Minus) => {
                self.at += 1;
                match self.unary()? {
                    PyNum::Int(n) => n
                        .checked_neg()
                        .map(PyNum::Int)
                        .ok_or_else(|| int_limit(&format!("-{n}"))),
                    PyNum::Float(f) => Ok(PyNum::Float(-f)),
                }
            }
            _ => self.pow(),
        }
    }

    fn pow(&mut self) -> Val {
        let base = match self.peek() {
            Some(Tok::Num(n)) => *n,
            _ => return Err(EvalError::SyntaxError),
        };
        self.at += 1;
        if self.peek() != Some(&Tok::Pow) {
            return Ok(base);
        }
        self.at += 1;
        let exp = self.unary()?;
        power(base, exp)
    }
}

/// `fractions.Fraction`: an exact rational, reduced, with a positive
/// denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fraction {
    num: i128,
    den: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

impl Fraction {
    /// `Fraction(num, den)`; `None` for a zero denominator
    /// (`ZeroDivisionError`).
    pub fn new(num: i128, den: i128) -> Option<Fraction> {
        if den == 0 {
            return None;
        }
        let g = gcd(num, den).max(1);
        let (mut n, mut d) = (num / g, den / g);
        if d < 0 {
            (n, d) = (n.checked_neg()?, d.checked_neg()?);
        }
        Some(Fraction { num: n, den: d })
    }

    /// The numerator.
    pub fn numerator(self) -> i128 {
        self.num
    }

    /// The (positive) denominator.
    pub fn denominator(self) -> i128 {
        self.den
    }

    /// `Fraction(x)` for a finite float: its exact value. `None` for a
    /// non-finite float, or (recorded: `INT_LIMIT`) one whose exact
    /// denominator is beyond `i128`.
    pub fn from_f64(x: f64) -> Option<Fraction> {
        if !x.is_finite() {
            return None;
        }
        if x == 0.0 {
            return Fraction::new(0, 1);
        }
        let bits = x.to_bits();
        let sign: i128 = if bits >> 63 == 0 { 1 } else { -1 };
        let exp = i32::try_from((bits >> 52) & 0x7ff).unwrap_or_default();
        let frac = i128::from(bits & ((1 << 52) - 1));
        let (mant, e) = if exp == 0 {
            (frac, -1074)
        } else {
            (frac | (1 << 52), exp - 1075)
        };
        let limit = || {
            record_int_limit(&repr_float(x));
            None
        };
        if e >= 0 {
            return match u32::try_from(e).ok().and_then(|e| 2i128.checked_pow(e)) {
                Some(p) => mant.checked_mul(p).and_then(|n| Fraction::new(sign * n, 1)),
                None => limit(),
            };
        }
        match u32::try_from(-e).ok().and_then(|e| 2i128.checked_pow(e)) {
            Some(p) => Fraction::new(sign * mant, p),
            None => limit(),
        }
    }

    /// `limit_denominator(max_den)` (CPython's algorithm): the closest
    /// fraction with a denominator at most `max_den` (1 000 000 by default
    /// in Python).
    pub fn limit_denominator(self, max_den: i128) -> Fraction {
        if self.den <= max_den {
            return self;
        }
        let (mut p0, mut q0, mut p1, mut q1) = (0i128, 1i128, 1i128, 0i128);
        let (mut n, mut d) = (self.num, self.den);
        loop {
            let a = n.div_euclid(d);
            let q2 = q0 + a * q1;
            if q2 > max_den {
                break;
            }
            (p0, q0, p1, q1) = (p1, q1, p0 + a * p1, q2);
            (n, d) = (d, n - a * d);
            if d == 0 {
                break;
            }
        }
        let k = (max_den - q0) / q1;
        let b1 = Fraction::new(p0 + k * p1, q0 + k * q1).unwrap_or(self);
        let b2 = Fraction::new(p1, q1).unwrap_or(self);
        // the closer bound; on a tie, bound2 (Python: `if abs(bound2 - self)
        // <= abs(bound1 - self)`), exact while the products fit i128
        if let (Some(d2), Some(d1)) = (self.distance(b2), self.distance(b1))
            && let (Some(l), Some(r)) = (d2.0.checked_mul(d1.1), d1.0.checked_mul(d2.1))
        {
            return if l <= r { b2 } else { b1 };
        }
        let x = self.to_f64();
        if (b2.to_f64() - x).abs() <= (b1.to_f64() - x).abs() {
            b2
        } else {
            b1
        }
    }

    /// `|b - self|` as (numerator, denominator), when it fits.
    fn distance(self, b: Fraction) -> Option<(i128, i128)> {
        let n = b
            .num
            .checked_mul(self.den)?
            .checked_sub(self.num.checked_mul(b.den)?)?;
        Some((n.checked_abs()?, b.den.checked_mul(self.den)?))
    }

    /// The nearest float.
    #[allow(clippy::cast_precision_loss)]
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl std::fmt::Display for Fraction {
    /// `str(Fraction)`: `n` or `n/d`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

#[cfg(test)]
mod tests;
