//! Rate pacer as pure logic, driven by a virtual clock (AC 1).
//!
//! No sockets and no `sleep`: the pacer computes the send deadlines for a
//! configured packet rate, and the tests drive it with a virtual clock. The
//! sender binary is expected to sleep until these deadlines and to print the
//! resulting [`crate::summary::RunSummary`].

/// Nanoseconds per second.
pub const NS_PER_SEC: u64 = 1_000_000_000;

/// A fixed-rate send schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacerConfig {
    /// Packets per second; 2 000 at the default 0.5 ms slice (C-02).
    pub rate_pps: u64,
}

impl PacerConfig {
    /// Configure a pacer. The rate must be non-zero.
    pub fn new(rate_pps: u64) -> Self {
        let _ = rate_pps;
        todo!("validate rate and build the pacer configuration")
    }
}

/// Produces absolute send times relative to the start of a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatePacer {
    // Skeleton: fields are the coder's choice.
    config: PacerConfig,
}

impl RatePacer {
    /// A pacer for `rate_pps` packets per second.
    pub fn new(rate_pps: u64) -> Self {
        let _ = rate_pps;
        todo!("create a rate pacer")
    }

    /// The configured rate.
    pub fn rate_pps(&self) -> u64 {
        todo!("configured packets per second")
    }

    /// Absolute send time, in ns since the run start, for packet `index`
    /// (0-based). Packet 0 is due at t = 0.
    pub fn deadline_ns(&self, index: u64) -> u64 {
        let _ = index;
        todo!("send deadline for a 0-based packet index")
    }

    /// The virtual-clock schedule for `count` packets: exactly `count`
    /// deadlines, strictly increasing.
    pub fn schedule_ns(&self, count: u64) -> Vec<u64> {
        let _ = count;
        todo!("schedule count deadlines on the virtual clock")
    }

    /// Advance the virtual clock: return the next deadline and count one more
    /// packet as sent.
    pub fn next_deadline_ns(&mut self) -> u64 {
        todo!("next deadline on the virtual clock")
    }

    /// How many deadlines [`RatePacer::next_deadline_ns`] has returned.
    pub fn packets_sent(&self) -> u64 {
        todo!("number of packets sent so far")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pacer_steps_by_half_a_millisecond() {
        // C-02: 2 000 slices/s = one packet every 0.5 ms.
        let pacer = RatePacer::new(2_000);
        assert_eq!(pacer.rate_pps(), 2_000);
        assert_eq!(
            pacer.schedule_ns(5),
            vec![0, 500_000, 1_000_000, 1_500_000, 2_000_000]
        );
        assert_eq!(pacer.deadline_ns(3), 1_500_000);
    }

    #[test]
    fn pacer_honours_other_configured_rates() {
        assert_eq!(
            RatePacer::new(1_000).schedule_ns(3),
            vec![0, 1_000_000, 2_000_000]
        );
        assert_eq!(
            RatePacer::new(4_000).schedule_ns(3),
            vec![0, 250_000, 500_000]
        );
    }

    #[test]
    fn next_deadline_advances_a_virtual_clock() {
        let mut pacer = RatePacer::new(2_000);
        let mut observed = Vec::new();
        for _ in 0..4 {
            observed.push(pacer.next_deadline_ns());
        }
        assert_eq!(observed, vec![0, 500_000, 1_000_000, 1_500_000]);
        assert_eq!(pacer.packets_sent(), 4);
    }

    #[test]
    fn default_schedule_covers_50_seconds_for_100_000_packets() {
        // AC 1: 100 000 datagrams at 2 000 packets/s is a 50 s run.
        let schedule = RatePacer::new(2_000).schedule_ns(100_000);
        assert_eq!(schedule.len(), 100_000);
        assert_eq!(schedule.first(), Some(&0));
        assert_eq!(schedule.last(), Some(&49_999_500_000));
        let elapsed_s = *schedule.last().expect("non-empty schedule") as f64 / NS_PER_SEC as f64;
        assert!(
            (elapsed_s - 50.0).abs() <= 1.0,
            "run length {elapsed_s} s is not 50 s ± 1 s"
        );
        for pair in schedule.windows(2) {
            assert_eq!(pair[1] - pair[0], 500_000, "deadlines are 0.5 ms apart");
        }
    }

    #[test]
    fn pacer_config_carries_the_rate() {
        let config = PacerConfig::new(2_000);
        assert_eq!(config.rate_pps, 2_000);
    }
}
