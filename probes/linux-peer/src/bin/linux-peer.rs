//! Sender binary for the Linux peer.
//!
//! Ticket 001-01: sends IPv4 UDP datagrams to a given address/port at a
//! configurable rate for a configurable count, then prints achieved rate,
//! count, UDP payload length and assumed wire size.
//!
//! Usage: `linux-peer <dest> [--rate <pps>] [--count <n>] [--payload-len <bytes>]`
//! Defaults: 127.0.0.1:9000, 2 000 pps (C-02), 100 000 datagrams, 1 252-byte
//! UDP payload (C-137). IPv4 only (C-100).

use std::net::UdpSocket;
use std::time::{Duration, Instant};

use linux_peer::config::PeerConfig;
use linux_peer::pacer::RatePacer;
use linux_peer::payload::encode;
use linux_peer::summary::RunSummary;

fn main() {
    // No arguments at all: run with the defaults (127.0.0.1:9000, 2 000 pps,
    // 100 000 datagrams, 1 252-byte payload). Anything else goes through the
    // parser, which still requires a destination.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = if args.is_empty() {
        PeerConfig::default()
    } else {
        PeerConfig::from_args(args).unwrap_or_else(|err| {
            eprintln!("linux-peer: {err:?}");
            eprintln!(
                "usage: linux-peer <dest> [--rate <pps>] [--count <n>] \
                 [--payload-len <bytes>]"
            );
            std::process::exit(2);
        })
    };

    // Unconnected socket + send_to: a silent destination (no listener) must
    // not turn ICMP port-unreachable into send errors mid-run.
    let socket = UdpSocket::bind("0.0.0.0:0").expect("bind an ephemeral UDP socket");
    println!(
        "sending {} datagrams to {} at {} pps, UDP payload {} bytes",
        config.count, config.dest, config.rate_pps, config.udp_payload_len,
    );

    let start = Instant::now();
    let mut pacer = RatePacer::new(config.rate_pps);
    for sequence in 0..config.count {
        // Absolute-deadline pacing (the POC's fixed sleeps were the bug):
        // packet `sequence` is due at start + deadline_ns(sequence); sleep
        // until that instant, never a fixed offset.
        let deadline = start + Duration::from_nanos(pacer.next_deadline_ns());
        pace_until(deadline);

        let send_time_ns = start.elapsed().as_nanos() as u64;
        let payload = encode(sequence, send_time_ns, config.udp_payload_len);
        socket
            .send_to(&payload, config.dest)
            .expect("send a datagram");
    }
    let elapsed_ns = start.elapsed().as_nanos() as u64;

    let summary = RunSummary::new(config.count, config.udp_payload_len, elapsed_ns);
    println!("{}", summary.render());
}

/// Sleep, then spin, until `deadline`. The sleep leaves a margin so timer
/// slack cannot make the packet late; the spin covers the last sliver and
/// keeps the schedule on its absolute deadlines.
fn pace_until(deadline: Instant) {
    const SPIN_MARGIN: Duration = Duration::from_micros(200);
    loop {
        let now = Instant::now();
        if now >= deadline {
            return;
        }
        let remaining = deadline - now;
        if remaining > SPIN_MARGIN {
            std::thread::sleep(remaining - SPIN_MARGIN);
        } else {
            std::thread::yield_now();
        }
    }
}
