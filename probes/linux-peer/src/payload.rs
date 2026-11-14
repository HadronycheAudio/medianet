//! Probe payload encoding and decoding.
//!
//! Ticket 001-01: each datagram carries a 64-bit sequence number and a send
//! time; the UDP payload defaults to 1 252 bytes (C-137) and is configurable.
//! AC 3: sequence numbers are contiguous from 0 to count − 1, and the decoder
//! used by the macOS and Windows probes can detect gaps.

/// Default UDP payload length: the C-137 cap (C-06 / C-100 fallback gives a
/// 1 280-byte IPv4 datagram and 1 252 bytes of UDP payload).
pub const DEFAULT_UDP_PAYLOAD_LEN: usize = 1_252;

/// Bytes at the start of every probe payload before the zero-filled body:
/// 8 bytes of sequence number plus 8 bytes of send time.
pub const HEADER_LEN: usize = 16;

/// A decoded probe payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedPayload {
    /// 64-bit sequence number, contiguous from 0 for a clean run (AC 3).
    pub sequence: u64,
    /// Sender's stamp for this packet, in nanoseconds.
    pub send_time_ns: u64,
}

/// Why a buffer is not a probe payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The buffer is shorter than the fixed header.
    TooShort { len: usize, min: usize },
}

/// Encode one probe payload of exactly `len` bytes.
///
/// The first [`HEADER_LEN`] bytes carry the 64-bit sequence number and the
/// 64-bit send time; the rest is padding up to the configured UDP payload
/// length. `len` must be at least [`HEADER_LEN`].
pub fn encode(sequence: u64, send_time_ns: u64, len: usize) -> Vec<u8> {
    let _ = (sequence, send_time_ns, len);
    todo!("encode sequence + send time into a len-byte UDP payload")
}

/// Decode a received UDP payload. Trailing padding is ignored.
pub fn decode(buf: &[u8]) -> Result<DecodedPayload, DecodeError> {
    let _ = buf;
    todo!("decode the 64-bit sequence number and send time")
}

/// What one observation meant to the gap detector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceObservation {
    /// The observed sequence number.
    pub sequence: u64,
    /// Sequence numbers missing immediately before this one.
    pub gap: u64,
    /// This sequence number had already been seen.
    pub duplicate: bool,
    /// This sequence number is below the highest seen so far.
    pub out_of_order: bool,
}

/// Tracks the sequence numbers delivered to a receiver and detects gaps
/// (AC 3). Pure logic: it takes decoded sequence numbers, no network.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequenceTracker {
    // Skeleton: fields are the coder's choice; keep the accessors below.
    _skeleton: (),
}

impl SequenceTracker {
    /// A tracker that expects the stream to start at sequence number 0.
    pub fn new() -> Self {
        todo!("new gap detector")
    }

    /// Feed one decoded sequence number and learn what it means.
    pub fn observe(&mut self, sequence: u64) -> SequenceObservation {
        let _ = sequence;
        todo!("observe a sequence number and report gap/duplicate")
    }

    /// Total sequence numbers missing so far.
    pub fn lost(&self) -> u64 {
        todo!("total lost sequence numbers")
    }

    /// Number of distinct sequence numbers received.
    pub fn received(&self) -> u64 {
        todo!("number of packets received")
    }

    /// Number of duplicate sequence numbers seen.
    pub fn duplicates(&self) -> u64 {
        todo!("number of duplicate sequence numbers")
    }

    /// Highest sequence number seen, if any.
    pub fn highest(&self) -> Option<u64> {
        todo!("highest sequence number seen")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_payload_length_is_1252_bytes() {
        // C-137: 1 152 B audio + at most 100 B headers and checksum.
        assert_eq!(DEFAULT_UDP_PAYLOAD_LEN, 1_252);
        let buf = encode(0, 0, DEFAULT_UDP_PAYLOAD_LEN);
        assert_eq!(buf.len(), 1_252);
    }

    #[test]
    fn payload_length_is_configurable() {
        for len in [HEADER_LEN, 64, 512, 1_024, 1_252, 1_400] {
            assert_eq!(encode(7, 8, len).len(), len, "configured length {len}");
        }
    }

    #[test]
    fn sequence_and_send_time_roundtrip() {
        let buf = encode(1_234, 5_000_000_123, DEFAULT_UDP_PAYLOAD_LEN);
        let decoded = decode(&buf).expect("a full payload decodes");
        assert_eq!(
            decoded,
            DecodedPayload {
                sequence: 1_234,
                send_time_ns: 5_000_000_123,
            }
        );
    }

    #[test]
    fn sequence_number_is_64_bit() {
        for sequence in [0, 1, 1 << 32, u64::MAX - 1, u64::MAX] {
            let buf = encode(sequence, 0, HEADER_LEN);
            assert_eq!(
                decode(&buf).expect("header-only payload").sequence,
                sequence
            );
        }
    }

    #[test]
    fn send_time_is_a_64_bit_nanosecond_stamp() {
        for send_time_ns in [0, 1, 1 << 32, u64::MAX] {
            let buf = encode(0, send_time_ns, HEADER_LEN);
            assert_eq!(
                decode(&buf).expect("header-only payload").send_time_ns,
                send_time_ns
            );
        }
    }

    #[test]
    fn decoder_rejects_truncated_payloads() {
        assert_eq!(
            decode(&[]),
            Err(DecodeError::TooShort {
                len: 0,
                min: HEADER_LEN,
            })
        );
        assert_eq!(
            decode(&[0u8; HEADER_LEN - 1]),
            Err(DecodeError::TooShort {
                len: HEADER_LEN - 1,
                min: HEADER_LEN,
            })
        );
        assert!(decode(&[0u8; HEADER_LEN]).is_ok());
    }

    #[test]
    fn contiguous_sequence_numbers_have_no_gaps() {
        // AC 3: sequence numbers 0..count-1 are contiguous and gap-free.
        let mut tracker = SequenceTracker::new();
        for sequence in 0..1_000u64 {
            let buf = encode(sequence, 1_000 + sequence, DEFAULT_UDP_PAYLOAD_LEN);
            let decoded = decode(&buf).expect("encoded payload decodes");
            assert_eq!(decoded.sequence, sequence);
            let observed = tracker.observe(decoded.sequence);
            assert_eq!(observed.gap, 0, "unexpected gap before {sequence}");
            assert!(!observed.duplicate);
            assert!(!observed.out_of_order);
        }
        assert_eq!(tracker.received(), 1_000);
        assert_eq!(tracker.lost(), 0);
        assert_eq!(tracker.duplicates(), 0);
        assert_eq!(tracker.highest(), Some(999));
    }

    #[test]
    fn decoder_detects_dropped_sequence_numbers() {
        // AC 3: the macOS / Windows probe decoder detects gaps.
        let mut tracker = SequenceTracker::new();
        for sequence in 0..10u64 {
            tracker.observe(sequence);
        }
        let observed = tracker.observe(13);
        assert_eq!(observed.gap, 3, "10, 11 and 12 were lost");
        assert!(!observed.duplicate);
        assert_eq!(tracker.lost(), 3);
        assert_eq!(tracker.received(), 11);
        assert_eq!(tracker.highest(), Some(13));
    }

    #[test]
    fn decoder_detects_gaps_through_decoded_payloads() {
        let mut tracker = SequenceTracker::new();
        let sent = 0..100u64;
        let lost: [u64; 3] = [17, 42, 99];
        for sequence in sent {
            if lost.contains(&sequence) {
                continue;
            }
            let buf = encode(sequence, 0, DEFAULT_UDP_PAYLOAD_LEN);
            let decoded = decode(&buf).expect("encoded payload decodes");
            tracker.observe(decoded.sequence);
        }
        assert_eq!(tracker.received(), 97);
        assert_eq!(tracker.lost(), 3);
    }

    #[test]
    fn duplicate_sequence_number_is_flagged() {
        let mut tracker = SequenceTracker::new();
        tracker.observe(0);
        tracker.observe(1);
        let observed = tracker.observe(1);
        assert!(observed.duplicate);
        assert!(observed.out_of_order);
        assert_eq!(tracker.received(), 2);
        assert_eq!(tracker.duplicates(), 1);
        assert_eq!(tracker.lost(), 0);
    }
}
