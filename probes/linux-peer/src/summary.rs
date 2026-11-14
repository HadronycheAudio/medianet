//! End-of-run summary: count, achieved rate, UDP payload length and assumed
//! wire size (AC 1 and AC 2; C-20, C-137).

/// C-20: the wire size of one default packet is about 1 320 bytes including
/// all Ethernet overhead (preamble, inter-frame gap and FCS included).
pub const C20_WIRE_SIZE_BYTES: usize = 1_320;

/// The figures the sender prints at the end of a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunSummary {
    /// Datagrams sent, exactly the configured count (AC 1).
    pub count: u64,
    /// UDP payload length used, 1 252 bytes by default (C-137).
    pub udp_payload_len: usize,
    /// Assumed on-the-wire size per packet, about 1 320 bytes (C-20).
    pub wire_size: usize,
    /// Wall-clock duration of the run, in nanoseconds.
    pub elapsed_ns: u64,
}

impl RunSummary {
    /// Build a summary from a virtual-clock send schedule, as produced by
    /// [`crate::pacer::RatePacer::schedule_ns`]. The run starts at t = 0 and
    /// the elapsed time is the last deadline.
    pub fn from_schedule(schedule: &[u64], udp_payload_len: usize) -> Self {
        let elapsed_ns = schedule.last().copied().unwrap_or(0);
        Self::new(schedule.len() as u64, udp_payload_len, elapsed_ns)
    }

    /// Build a summary from measured values.
    pub fn new(count: u64, udp_payload_len: usize, elapsed_ns: u64) -> Self {
        Self {
            count,
            udp_payload_len,
            wire_size: estimated_wire_size(udp_payload_len),
            elapsed_ns,
        }
    }

    /// Achieved packet rate in packets per second.
    pub fn achieved_rate_pps(&self) -> f64 {
        // Aggregate rate over the whole run: every packet counted, elapsed
        // wall-clock time.
        self.count as f64 / (self.elapsed_ns as f64 / NS_PER_SEC_F64)
    }

    /// Whether the achieved rate is within `tolerance` (a fraction, 0.02 for
    /// ±2 %) of `target_pps`.
    pub fn achieved_rate_in_tolerance(&self, target_pps: f64, tolerance: f64) -> bool {
        // Compare packet intervals, not aggregate rates: the run spaced
        // `count` packets over `elapsed_ns`, i.e. `count − 1` intervals. The
        // aggregate rate (above) dilutes a pacing error across one extra
        // packet; the interval shows it in full.
        if target_pps <= 0.0 || self.count < 2 {
            return false;
        }
        let measured_interval_ns = self.elapsed_ns as f64 / (self.count - 1) as f64;
        let target_interval_ns = NS_PER_SEC_F64 / target_pps;
        (measured_interval_ns - target_interval_ns).abs() <= target_interval_ns * tolerance
    }

    /// The text the sender prints at the end of a run: achieved rate, count,
    /// UDP payload length and assumed wire size (ticket 001-01, Work).
    pub fn render(&self) -> String {
        let elapsed_s = self.elapsed_ns as f64 / NS_PER_SEC_F64;
        format!(
            "sent {} datagrams in {:.6} s: achieved rate {:.1} pps, \
             UDP payload {} bytes, assumed wire size {} bytes",
            self.count, elapsed_s, self.achieved_rate_pps(), self.udp_payload_len, self.wire_size,
        )
    }
}

/// Nanoseconds per second, as a float for the rate arithmetic.
const NS_PER_SEC_F64: f64 = 1_000_000_000.0;

/// Assumed on-the-wire size for a UDP payload of `udp_payload_len` bytes:
/// the payload plus IPv4, UDP and all Ethernet overhead (C-20). At the
/// 1 252-byte default this is about 1 320 bytes.
pub fn estimated_wire_size(udp_payload_len: usize) -> usize {
    // Per-packet overhead, all included in the C-20 figure:
    //   12  inter-frame gap
    //    8  preamble
    //   14  Ethernet header (MACs + EtherType)
    //   20  IPv4 header (C-100: no IPv6, no options)
    //    8  UDP header
    //    4  FCS
    //  = 66 bytes on top of the UDP payload.
    udp_payload_len + 66
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pacer::{NS_PER_SEC, RatePacer};
    use crate::payload::DEFAULT_UDP_PAYLOAD_LEN;

    #[test]
    fn default_summary_reports_1252_byte_payload_and_c20_wire_size() {
        // AC 2: the printed UDP payload length is 1 252 bytes and the printed
        // wire size is the C-20 figure, about 1 320 bytes.
        let summary = RunSummary::new(100_000, DEFAULT_UDP_PAYLOAD_LEN, 50 * NS_PER_SEC);
        assert_eq!(summary.udp_payload_len, 1_252);
        assert_eq!(summary.count, 100_000);
        assert!(
            (summary.wire_size as i64 - C20_WIRE_SIZE_BYTES as i64).abs() <= 4,
            "wire size {} is not the C-20 figure of about {C20_WIRE_SIZE_BYTES}",
            summary.wire_size
        );
        let rendered = summary.render();
        assert!(
            rendered.contains("1252"),
            "summary must print the payload length: {rendered}"
        );
        assert!(
            rendered.contains(&summary.wire_size.to_string()),
            "summary must print the assumed wire size: {rendered}"
        );
    }

    #[test]
    fn c20_wire_size_is_about_1320_and_scales_with_payload() {
        let default_wire = estimated_wire_size(DEFAULT_UDP_PAYLOAD_LEN);
        assert!(
            (default_wire as i64 - C20_WIRE_SIZE_BYTES as i64).abs() <= 4,
            "estimated wire size for a default packet is {default_wire}, not about {C20_WIRE_SIZE_BYTES}"
        );
        // All per-packet overhead is constant, so a smaller payload loses
        // exactly the difference in payload bytes.
        let small_wire = estimated_wire_size(512);
        assert_eq!(default_wire - small_wire, DEFAULT_UDP_PAYLOAD_LEN - 512);
    }

    #[test]
    fn achieved_rate_is_2000_pps_for_default_config() {
        // AC 1: 100 000 packets in 50 s is 2 000 packets/s.
        let summary = RunSummary::new(100_000, DEFAULT_UDP_PAYLOAD_LEN, 50 * NS_PER_SEC);
        assert!(
            (summary.achieved_rate_pps() - 2_000.0).abs() < 0.001,
            "achieved rate {}",
            summary.achieved_rate_pps()
        );
        assert!(summary.achieved_rate_in_tolerance(2_000.0, 0.02));
        assert!(!summary.achieved_rate_in_tolerance(2_000.0, 0.000_001));
    }

    #[test]
    fn schedule_summary_keeps_count_and_rate_within_two_percent() {
        // AC 1 measured by the program's own summary, over the virtual clock.
        let schedule = RatePacer::new(2_000).schedule_ns(100_000);
        let summary = RunSummary::from_schedule(&schedule, DEFAULT_UDP_PAYLOAD_LEN);
        assert_eq!(summary.count, 100_000);
        assert!(summary.achieved_rate_in_tolerance(2_000.0, 0.02));
        assert_eq!(summary.elapsed_ns, 49_999_500_000);
        let elapsed_s = summary.elapsed_ns as f64 / NS_PER_SEC as f64;
        assert!((elapsed_s - 50.0).abs() <= 1.0, "run length {elapsed_s} s");
    }

    #[test]
    fn rendered_summary_contains_all_four_printed_figures() {
        // Ticket 001-01, Work: prints achieved rate, count, UDP payload length
        // and assumed wire size at the end.
        let summary = RunSummary::new(100_000, DEFAULT_UDP_PAYLOAD_LEN, 50 * NS_PER_SEC);
        let rendered = summary.render();
        assert!(
            rendered.contains("100000")
                || rendered.contains("100 000")
                || rendered.contains("100_000"),
            "count missing from summary: {rendered}"
        );
        assert!(
            rendered.contains("2000") || rendered.contains("2 000") || rendered.contains("2_000"),
            "achieved rate missing from summary: {rendered}"
        );
        assert!(
            rendered.contains("1252"),
            "payload length missing: {rendered}"
        );
        assert!(
            rendered.contains(&summary.wire_size.to_string()),
            "wire size missing: {rendered}"
        );
    }
}
