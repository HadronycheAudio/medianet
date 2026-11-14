//! Acceptance tests that need the crate root, the file layout or a child
//! process: the capture procedure (AC 4) and the fuzz target (AC 5).

use std::path::Path;
use std::process::Command;

use linux_peer::fuzz::FUZZ_SECONDS_PER_CHANGE;
use linux_peer::payload::DEFAULT_UDP_PAYLOAD_LEN;
use linux_peer::summary::RunSummary;

#[test]
fn ac4_capture_md_gives_one_tcpdump_command_with_an_ipv4_fragment_filter() {
    // C-133: the probe is a throwaway probe under probes/.
    let probe_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        probe_dir.file_name().and_then(|name| name.to_str()),
        Some("linux-peer"),
        "probe must live in probes/linux-peer/ (C-133)"
    );
    assert_eq!(
        probe_dir
            .parent()
            .and_then(|dir| dir.file_name())
            .and_then(|name| name.to_str()),
        Some("probes"),
        "probe must live in probes/linux-peer/ (C-133)"
    );

    let capture = probe_dir.join("CAPTURE.md");
    assert!(
        capture.is_file(),
        "AC 4: missing {}; the capture command is a ticket deliverable",
        capture.display()
    );
    let text = std::fs::read_to_string(&capture).expect("CAPTURE.md is readable");

    // AC 4: one command that lists any IPv4 fragment received from the probe
    // host. Whitespace-insensitive so both `ip[6:2] & 0x3fff != 0` and
    // `ip[6:2]&0x3fff!=0` count.
    let fragment_command = text.lines().find(|line| {
        let compact: String = line.split_whitespace().collect();
        compact.contains("tcpdump") && compact.contains("ip[6:2]&0x3fff!=0")
    });
    assert!(
        fragment_command.is_some(),
        "AC 4: CAPTURE.md must give a tcpdump command with the IPv4 fragment filter \
         `ip[6:2] & 0x3fff != 0`:\n{text}"
    );
    let command = fragment_command.expect("checked above");
    assert!(
        command.contains("-w") || command.contains("-c") || command.contains("fragments"),
        "AC 4: the fragment command must be runnable and show/save what it captured: {command}"
    );

    // Ticket 001-01: how to save the .pcap with results under
    // doc/design/reports/ (C-130).
    assert!(
        text.contains("doc/design/reports/"),
        "C-130: CAPTURE.md must say to save results under doc/design/reports/"
    );
    assert!(
        text.contains(".pcap"),
        "CAPTURE.md must produce a .pcap file"
    );
}

#[test]
fn ac5_fuzz_target_runs_for_a_short_fixed_duration_without_a_crash() {
    // AC 5 / C-95: the payload-decoder fuzz target is runnable from the
    // command line for a fixed duration. The per-change budget is 60 s; this
    // test uses 1 s so the suite stays fast and unprivileged (C-146).
    assert_eq!(FUZZ_SECONDS_PER_CHANGE, 60);

    let fuzz_bin = env!("CARGO_BIN_EXE_fuzz-decode");
    let output = Command::new(fuzz_bin)
        .args(["--duration-secs", "1"])
        .output()
        .expect("spawn the fuzz target");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "AC 5: fuzz target failed with {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        status = output.status
    );
    assert!(
        stdout.contains("executions"),
        "AC 5: the fuzz target must report how many inputs it executed:\n{stdout}"
    );
}

#[test]
fn ac1_and_ac2_default_run_summary_over_the_virtual_clock() {
    // The whole default run as pure logic: C-02 rate, ticket count, C-137
    // payload length, C-20 wire size and the C-20 report text.
    use linux_peer::config::PeerConfig;
    use linux_peer::pacer::RatePacer;

    let config = PeerConfig::default();
    let schedule = RatePacer::new(config.rate_pps).schedule_ns(config.count);
    let summary = RunSummary::from_schedule(&schedule, config.udp_payload_len);

    assert_eq!(summary.count, 100_000, "AC 1: exactly the configured count");
    assert!(
        summary.achieved_rate_in_tolerance(2_000.0, 0.02),
        "AC 1: achieved rate {} is not 2 000 pps ± 2 %",
        summary.achieved_rate_pps()
    );
    assert_eq!(
        summary.udp_payload_len, DEFAULT_UDP_PAYLOAD_LEN,
        "AC 2: default UDP payload length"
    );
    assert!(
        (summary.wire_size as i64 - 1_320).abs() <= 4,
        "AC 2: wire size {} is not the C-20 figure of about 1 320 bytes",
        summary.wire_size
    );
    let text = summary.render();
    assert!(
        text.contains("1252"),
        "AC 2: summary must print 1 252:\n{text}"
    );
    assert!(
        text.contains(&summary.wire_size.to_string()),
        "AC 2: summary must print the wire size:\n{text}"
    );
}
