//! `linux-peer` — throwaway IPv4 UDP peer traffic generator for epic 001.
//!
//! This is the RED-phase skeleton of ticket 001-01: module structure, types,
//! signatures and tests only. Every `todo!()` body is the coder's work; the
//! tests in this crate are the acceptance contract.
//!
//! Cited constants:
//! - C-02: default slice 48 frames = 0.5 ms = 2 000 slices/s.
//! - C-04: 6 channels × 192 bytes = 1 152 bytes of audio.
//! - C-20: about 1 320 bytes on the wire per default packet (all Ethernet
//!   overhead, preamble, inter-frame gap and FCS included).
//! - C-95: fuzz budget, 60 s on every change and 1 hour nightly.
//! - C-100: IPv4 only in v1; 1 280-byte fallback gives 1 252 B UDP payload.
//! - C-133: throwaway probes live in `probes/`.
//! - C-137: a default slice must fit 1 252 bytes of UDP payload.

pub mod config;
pub mod fuzz;
pub mod pacer;
pub mod payload;
pub mod summary;
