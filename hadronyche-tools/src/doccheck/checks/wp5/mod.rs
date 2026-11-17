//! The code the four WP5 ports (004-06, 004-08, 004-09, 004-10) share, in
//! one module, compiled into each port with `#[path = "../wp5/mod.rs"]` so
//! that no port imports another port. Every item reproduces the scripts'
//! identical helpers; it is a candidate for the library:
//!
//! * [`py`] and [`pyuni`]: Python 3.10 text semantics (universal newlines,
//!   `\w`/`\s`/`\d` at Unicode 13 through a shadow text, `re.I` folding of
//!   `İ`/`ı`, look-behind emulation, `int()` and `float()`, `repr`);
//! * [`big`]: Python's unbounded `int`;
//! * [`pytoml`]: the scripts' TOML subset reader (one file, swappable for a
//!   library `TomlReader`);
//! * [`toml`]: wire.toml access with Python container semantics;
//! * [`md`]: `split_row`, `parse_tables`, `heading_index`, `strip_fences`,
//!   `blocks`, `norm_name`, `width_to_bits`, `bit_positions`,
//!   `is_field_table`, the placeholder rule and the every-block tag rule;
//! * [`members`]: the 13-name enumeration as coded (>= 10 found replace it);
//! * [`reasons`]: member and non-member reason tokens with the negation
//!   guard.

pub(crate) mod big;
pub(crate) mod md;
pub(crate) mod members;
pub(crate) mod py;
pub(crate) mod pytoml;
mod pyuni;
pub(crate) mod reasons;
pub(crate) mod toml;
