//! Sender binary for the Linux peer.
//!
//! Ticket 001-01: sends IPv4 UDP datagrams to a given address/port at a
//! configurable rate for a configurable count, then prints achieved rate,
//! count, UDP payload length and assumed wire size.
//!
//! RED-phase skeleton: the coder wires `PeerConfig::from_args`,
//! `RatePacer` and the socket loop here.

fn main() {
    todo!("parse PeerConfig, send datagrams at the paced deadlines, print RunSummary")
}
