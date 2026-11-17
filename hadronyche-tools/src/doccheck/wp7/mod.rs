//! The shared code of the WP7 ports (004-16 to 004-20), in one place and
//! separate from every port, so it can move into the library:
//!
//! * [`py`] — Python semantics (the shadow text for `\w`, `\b`, `\s`, the
//!   `re.I` dotless i, `str` methods, unbounded `int`);
//! * [`md`] — the Markdown helpers of the 004-16/17/18 family (and the
//!   equivalent 004-19/20 ones);
//! * [`pytoml`] — a faithful port of the 004 family's TOML subset reader,
//!   with Python value semantics (isolated so a library `TomlReader` can
//!   replace it);
//! * [`common`] — the remaining helpers several of the five ports call.

pub mod common;
pub mod md;
pub mod py;
pub mod pytoml;
pub mod rejection_members;
