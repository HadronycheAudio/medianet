//! Wire layout of a `hadropoc` datagram, mirrored from the POC's `proto.rs`
//! (frozen POC, read-only): one fixed little-endian 88-byte header followed
//! by messages. The flood datagram replicates the sender's audio feed to a
//! mixer input: header plus one audio message of `frames` frames in
//! `channels` f32 channels (all-zero samples), the format the mixer's feed
//! handler accepts and mixes.
//!
//! Echo-only header fields (t2k, t2, t3, ack_t4) and the loss/drift fields
//! stay zero: the mixer overwrites them when it answers.

pub const HDR: usize = 88;
pub const MSG: usize = 8;
pub const MAGIC: u16 = 0x4844;
pub const VERSION: u8 = 11;
pub const K_AUDIO: u8 = 0;

/// Total length of one flood datagram: header, message header, payload.
pub fn datagram_len(frames: u16, channels: u8) -> usize {
    HDR + MSG + frames as usize * channels as usize * 4
}

/// The fields of the flood header that stay fixed across the run; the
/// per-datagram fields (`seq`, `sample_pos`, `t1`) are stamped by `encode`.
pub struct FloodHeader {
    pub session: u32,
    pub frames: u16,
    pub channels: u8,
    /// The POC stamps the sample rate here (sender.rs:309).
    pub path_delay: u32,
}

impl FloodHeader {
    /// Encode the fixed datagram header into `b[0..HDR]`, field offsets
    /// exactly as in the POC's `proto.rs` (`Header::encode`).
    pub fn encode(&self, b: &mut [u8], seq: u32, sample_pos: u64, t1: i64) {
        assert!(b.len() >= HDR, "header buffer too short");
        b[0..2].copy_from_slice(&MAGIC.to_le_bytes());
        b[2] = VERSION;
        b[3] = 0; // flags: neither F_ECHO nor F_FIXED
        b[4..8].copy_from_slice(&self.session.to_le_bytes());
        b[8..12].copy_from_slice(&seq.to_le_bytes());
        b[12..14].copy_from_slice(&self.frames.to_le_bytes());
        b[14] = self.channels;
        b[15] = 0; // lock
        b[16..24].copy_from_slice(&sample_pos.to_le_bytes());
        b[24..32].copy_from_slice(&t1.to_le_bytes());
        // bytes 32..64 (t2k, t2, t3, ack_t4) stay zero
        b[64..68].copy_from_slice(&0u32.to_le_bytes()); // ack_seq: 0, a foreign sender
        b[68..72].copy_from_slice(&0i32.to_le_bytes()); // tx_late
        b[72..76].copy_from_slice(&0u32.to_le_bytes()); // rx_lost
        // bytes 76..84 (drift_ppb, servo_err) stay zero
        b[84..88].copy_from_slice(&self.path_delay.to_le_bytes());
    }
}

/// Encode one audio message header at `b[HDR..HDR+MSG]`: kind `K_AUDIO`,
/// `channels` channels, `frames` frames, payload length implied by both, at
/// sample offset `offset` relative to the datagram's `sample_pos`.
pub fn encode_audio_msg(b: &mut [u8], channels: u8, frames: u16, offset: i16) {
    assert!(b.len() >= HDR + MSG, "message buffer too short");
    let at = HDR;
    b[at] = K_AUDIO << 4;
    b[at + 1] = channels;
    b[at + 2..at + 4].copy_from_slice(&frames.to_le_bytes());
    let len = frames as usize * channels as usize * 4;
    assert!(len <= u16::MAX as usize, "payload does not fit a message");
    b[at + 4..at + 6].copy_from_slice(&(len as u16).to_le_bytes());
    b[at + 6..at + 8].copy_from_slice(&offset.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_matches_poc_layout() {
        let mut b = vec![0u8; HDR];
        let h = FloodHeader {
            session: 0x5EC0F100,
            frames: 48,
            channels: 48,
            path_delay: 96_000,
        };
        h.encode(&mut b, 900_000_001, 1234, 0x0102030405060708);
        assert_eq!(&b[0..2], &MAGIC.to_le_bytes()); // magic at 0..2
        assert_eq!(b[2], VERSION); // version at 2
        assert_eq!(b[3], 0); // flags at 3
        assert_eq!(&b[4..8], &0x5EC0F100u32.to_le_bytes()); // session at 4..8
        assert_eq!(&b[8..12], &900_000_001u32.to_le_bytes()); // seq at 8..12
        assert_eq!(&b[12..14], &48u16.to_le_bytes()); // frames at 12..14
        assert_eq!(b[14], 48); // channels at 14
        assert_eq!(&b[16..24], &1234u64.to_le_bytes()); // sample_pos at 16..24
        assert_eq!(&b[24..32], &0x0102030405060708i64.to_le_bytes()); // t1 at 24..32
        assert_eq!(&b[72..76], &0u32.to_le_bytes()); // rx_lost at 72..76
        assert_eq!(&b[84..88], &96_000u32.to_le_bytes()); // path_delay at 84..88
    }

    #[test]
    fn audio_message_declares_its_payload() {
        let mut b = vec![0u8; HDR + MSG];
        encode_audio_msg(&mut b, 48, 48, 0);
        assert_eq!(b[HDR] >> 4, K_AUDIO);
        assert_eq!(b[HDR + 1], 48);
        assert_eq!(&b[HDR + 2..HDR + 4], &48u16.to_le_bytes()); // frames
        assert_eq!(&b[HDR + 4..HDR + 6], &9216u16.to_le_bytes()); // payload length
    }

    #[test]
    fn datagram_length() {
        assert_eq!(datagram_len(48, 48), HDR + MSG + 48 * 48 * 4);
        assert_eq!(datagram_len(0, 48), HDR + MSG);
    }
}
