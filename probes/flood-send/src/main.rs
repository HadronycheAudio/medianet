//! flood-send — throwaway probe for ticket 002-10 (flood test of the
//! unbounded drain, C-133). It sends well-formed `hadropoc` feed datagrams —
//! the exact wire format of the POC's sender→mixer audio feed, with its own
//! session and an advancing sequence number — to one mixer input at a
//! configurable rate, and prints the rate it actually achieved.
//!
//! Standard use, against a loopback run started with `--port P` (the mixer's
//! feed input is then UDP port P + 31; the mixer binds its clock, inputs and
//! outputs consecutively from P + 10 × <module index>):
//!
//! ```console
//! $ flood-send --target 127.0.0.1:45431 --rate 20000 --duration 28
//! ```

mod pacer;
mod wire;

use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Sample rate the POC stamps into `path_delay` (sender.rs:309).
const PATH_DELAY: u32 = 96_000;

struct Config {
    target: String,
    rate: f64,
    duration: f64,
    session: u32,
    seq_start: u32,
    frames: u16,
    channels: u8,
}

fn print_usage() {
    eprintln!(
        "usage: flood-send --target IP:PORT --rate PER_SECOND [--duration S] \
         [--session N] [--seq-start N] [--frames N] [--channels N]"
    );
}

fn parse_args(args: &[String]) -> Option<Config> {
    let mut cfg = Config {
        target: String::new(),
        rate: 0.0,
        duration: 30.0,
        session: 0x5EC0_F100,
        seq_start: 900_000_000,
        frames: 48,
        channels: 48,
    };
    let mut idx = 0;
    while idx + 1 < args.len() {
        let (key, value) = (args[idx].as_str(), args[idx + 1].as_str());
        match key {
            "--target" => cfg.target = value.to_string(),
            "--rate" => cfg.rate = value.parse().ok()?,
            "--duration" => cfg.duration = value.parse().ok()?,
            "--session" => cfg.session = value.parse().ok()?,
            "--seq-start" => cfg.seq_start = value.parse().ok()?,
            "--frames" => cfg.frames = value.parse().ok()?,
            "--channels" => cfg.channels = value.parse().ok()?,
            _ => return None,
        }
        idx += 2;
    }
    if cfg.target.is_empty() || cfg.rate <= 0.0 || cfg.duration <= 0.0 {
        return None;
    }
    Some(cfg)
}

/// Wall-clock nanoseconds, the timebase the POC stamps into `t1`.
fn epoch_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i64)
}

/// One paced datagram: re-stamp the volatile header fields and send it.
fn send_one(sock: &UdpSocket, buf: &mut [u8], cfg: &Config, seq: u32, sample_pos: u64) -> bool {
    let head = wire::FloodHeader {
        session: cfg.session,
        frames: cfg.frames,
        channels: cfg.channels,
        path_delay: PATH_DELAY,
    };
    head.encode(buf, seq, sample_pos, epoch_ns());
    wire::encode_audio_msg(buf, cfg.channels, cfg.frames, 0);
    sock.send_to(buf, &cfg.target).is_ok()
}

fn run(cfg: &Config) {
    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind ephemeral UDP socket");
    let len = wire::datagram_len(cfg.frames, cfg.channels);
    let mut buf = vec![0u8; len];
    let mut pacer = pacer::Pacer::new(cfg.rate);
    let deadline = Instant::now() + Duration::from_secs_f64(cfg.duration);
    let t0 = Instant::now();
    let mut next_report = t0 + Duration::from_secs(1);
    let (mut seq, mut sent, mut errors) = (cfg.seq_start, 0u64, 0u64);
    while Instant::now() < deadline {
        pacer.wait();
        let sample_pos = sent.wrapping_mul(cfg.frames as u64);
        if send_one(&sock, &mut buf, cfg, seq, sample_pos) {
            sent += 1;
        } else {
            errors += 1;
        }
        seq = seq.wrapping_add(1);
        if Instant::now() >= next_report {
            let el = t0.elapsed().as_secs_f64();
            println!(
                "flood: t={el:.1}s sent={sent} achieved={:.0} datagrams per second",
                sent as f64 / el
            );
            next_report += Duration::from_secs(1);
        }
    }
    let el = t0.elapsed().as_secs_f64();
    println!(
        "flood: sent {sent} datagrams in {el:.3}s, achieved {:.0} datagrams per second (target {:.0}), {errors} send errors",
        sent as f64 / el,
        cfg.rate
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cfg) = parse_args(&args) else {
        print_usage();
        std::process::exit(2);
    };
    run(&cfg);
}
