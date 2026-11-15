//! Wire layout of a `hadropoc` feed datagram, mirrored from the POC at the
//! `poc-untouched` tag (frozen POC, read-only): one fixed little-endian
//! 88-byte header (proto.rs `Header::encode`) followed by messages. The
//! flood datagram is what the POC's sender builds for the mixer's feed input
//! (sender.rs `prepare`, `fire`): the header, one audio message and one
//! controller message — the transport messages and musical event parts the
//! sender adds only intermittently are left out.
//!
//! Audio message (audio.rs `Synth::render`, `payload_len`): an 8-byte `Msg`
//! header, then `channels` one-byte channel ids (0, 1, 2, ...), then
//! `frames` rows of `channels` little-endian f32 samples — `channels * (1 +
//! frames * 4)` payload bytes, 9 264 for 48 channels of 48 frames. The ids
//! must be distinct or the mixer rejects the payload.
//!
//! Controller message (control.rs `span`, `render`): an 8-byte `Msg` header,
//! one controller id, then `span` f32 ticks at one fourth of the sample
//! rate — 1 + 13 * 4 = 53 payload bytes for 48 frames.
//!
//! Echo-only header fields (t2k, t2, ack_t4) and the loss/drift fields stay
//! zero: the mixer overwrites them when it answers.

use std::f64::consts::TAU;

pub const HDR: usize = 88;
pub const MSG: usize = 8;
pub const MAGIC: u16 = 0x4844;
pub const VERSION: u8 = 11;
pub const K_AUDIO: u8 = 0;
pub const K_CONTROL: u8 = 1;
/// The POC's controller clock is the audio clock divided by this.
pub const DIVIDER: usize = 4;
/// The POC's volume controller id; the flood uses another one so it cannot
/// overwrite the real sender's volume ring in the mixer (control.rs `store`
/// returns early for a payload without the volume controller).
pub const VOLUME: u8 = 0;
pub const FLOOD_CONTROLLER: u8 = VOLUME + 1;

const LOW_HZ: u64 = 20;
const HIGH_HZ: u64 = 25_000;

/// Size in bytes of an audio payload (audio.rs `payload_len`).
pub fn payload_len(channels: u8, frames: u16) -> usize {
    if frames == 0 {
        0
    } else {
        channels as usize * (1 + frames as usize * 4)
    }
}

/// Controller ticks of a block that starts on a tick boundary (control.rs
/// `span`): the ticks covering the block plus the one after it.
pub fn control_ticks(frames: u16) -> usize {
    if frames == 0 {
        return 0;
    }
    (frames as usize - 1) / DIVIDER + 2
}

/// Total length of one flood datagram: header, audio message, controller
/// message.
pub fn datagram_len(frames: u16, channels: u8) -> usize {
    HDR + MSG + payload_len(channels, frames) + MSG + 1 + control_ticks(frames) * 4
}

/// The fields of the flood header that stay fixed across the run; the
/// per-datagram fields (`seq`, `sample_pos`, `t1`) are stamped by `encode`.
pub struct FloodHeader {
    pub session: u32,
    pub ack_seq: u32,
    pub frames: u16,
    pub channels: u8,
    /// The POC stamps the sample rate here (sender.rs:309).
    pub path_delay: u32,
}

impl FloodHeader {
    /// Encode the datagram header into `b[0..HDR]`, field offsets exactly as
    /// in the POC's `proto.rs` (`Header::encode`).
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
        b[48..56].copy_from_slice(&t1.to_le_bytes()); // t3, as sender.rs:306
        // bytes 32..48 and 56..64 (t2k, t2, ack_t4) stay zero
        b[64..68].copy_from_slice(&self.ack_seq.to_le_bytes());
        b[68..72].copy_from_slice(&0i32.to_le_bytes()); // tx_late
        b[72..76].copy_from_slice(&0u32.to_le_bytes()); // rx_lost
        // bytes 76..84 (drift_ppb, servo_err) stay zero
        b[84..88].copy_from_slice(&self.path_delay.to_le_bytes());
    }
}

/// Encode a message header (proto.rs `Msg::encode`) at `b[0..MSG]`.
fn encode_msg(b: &mut [u8], kind: u8, channels: u8, count: u16, len: usize, offset: i16) {
    assert!(len <= u16::MAX as usize, "payload does not fit a message");
    b[0] = kind << 4;
    b[1] = channels;
    b[2..4].copy_from_slice(&count.to_le_bytes());
    b[4..6].copy_from_slice(&(len as u16).to_le_bytes());
    b[6..8].copy_from_slice(&offset.to_le_bytes());
}

/// Write the audio message for the block at media position 0, as the POC's
/// float `Synth::render` produces it (audio.rs): channel ids, then one sine
/// per channel spread from 20 Hz to 25 kHz, rounded to f32. Returns the
/// bytes written, `MSG + payload_len`.
pub fn encode_audio_msg(b: &mut [u8], channels: u8, frames: u16, rate: u32) -> usize {
    let len = payload_len(channels, frames);
    assert!(b.len() >= MSG + len, "audio buffer too short");
    encode_msg(b, K_AUDIO, channels, frames, len, 0);
    let ch = channels as usize;
    let pcm = &mut b[MSG..MSG + len];
    for (id, slot) in pcm[..ch].iter_mut().enumerate() {
        *slot = id as u8;
    }
    let spans = (channels.max(2) - 1) as u64;
    let amp = 1.0 / channels as f64;
    for c in 0..ch {
        let num = LOW_HZ * spans + c as u64 * (HIGH_HZ - LOW_HZ);
        let (ds, dc) = (TAU * num as f64 / (spans * rate as u64) as f64).sin_cos();
        let (mut s, mut co) = (0.0f64, 1.0f64); // phase 0 at position 0
        for f in 0..frames as usize {
            let o = ch + (f * ch + c) * 4;
            pcm[o..o + 4].copy_from_slice(&((amp * s) as f32).to_le_bytes());
            let next = s * dc + co * ds;
            co = co * dc - s * ds;
            s = next;
        }
    }
    MSG + len
}

/// Write the controller message for the block at media position 0 (a tick
/// boundary): `control_ticks` f32 values of the POC's 0..1 test ramp, on
/// `FLOOD_CONTROLLER`. Returns the bytes written, `MSG + 1 + ticks * 4`.
pub fn encode_control_msg(b: &mut [u8], frames: u16, rate: u32) -> usize {
    let ticks = control_ticks(frames);
    let len = 1 + ticks * 4;
    assert!(b.len() >= MSG + len, "control buffer too short");
    encode_msg(b, K_CONTROL, 1, ticks as u16, len, 0);
    b[MSG] = FLOOD_CONTROLLER;
    let period = (rate as u64 / DIVIDER as u64).max(1);
    for k in 0..ticks {
        let phase = k as u64 % (2 * period);
        let ramp = phase.min(2 * period - phase) as f32 / period as f32;
        let o = MSG + 1 + k * 4;
        b[o..o + 4].copy_from_slice(&ramp.to_le_bytes());
    }
    MSG + len
}

/// Build the static part of a flood datagram (everything after the header)
/// into `b[HDR..]`; returns the total datagram length.
pub fn encode_body(b: &mut [u8], channels: u8, frames: u16, rate: u32) -> usize {
    let at = HDR + encode_audio_msg(&mut b[HDR..], channels, frames, rate);
    at + encode_control_msg(&mut b[at..], frames, rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_matches_poc_layout() {
        let mut b = vec![0u8; HDR];
        let h = FloodHeader {
            session: 0x5EC0F100,
            ack_seq: 0x5EC0F100,
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
        assert_eq!(&b[48..56], &0x0102030405060708i64.to_le_bytes()); // t3 at 48..56
        assert_eq!(&b[64..68], &0x5EC0F100u32.to_le_bytes()); // ack_seq at 64..68
        assert_eq!(&b[72..76], &0u32.to_le_bytes()); // rx_lost at 72..76
        assert_eq!(&b[84..88], &96_000u32.to_le_bytes()); // path_delay at 84..88
    }

    #[test]
    fn audio_payload_is_the_pocs_length() {
        // audio.rs payload_len: channels * (1 + frames * 4), the channel-id
        // byte preceding each channel's samples.
        assert_eq!(payload_len(48, 48), 9264);
        assert_eq!(payload_len(48, 48), 48 + 48 * 48 * 4);
        assert_eq!(payload_len(48, 0), 0);
        let mut b = vec![0u8; MSG + 9264];
        assert_eq!(encode_audio_msg(&mut b, 48, 48, 96_000), MSG + 9264);
        assert_eq!(b[0] >> 4, K_AUDIO);
        assert_eq!(b[1], 48);
        assert_eq!(&b[2..4], &48u16.to_le_bytes()); // frames
        assert_eq!(&b[4..6], &9264u16.to_le_bytes()); // payload length
    }

    #[test]
    fn channel_ids_are_distinct_and_precede_the_samples() {
        let mut b = vec![0u8; MSG + 9264];
        encode_audio_msg(&mut b, 48, 48, 96_000);
        let ids: Vec<u8> = b[MSG..MSG + 48].to_vec();
        assert_eq!(ids, (0..48).collect::<Vec<u8>>());
        // Channel 0 is 20 Hz from phase 0: the first frame is exactly 0.0,
        // the second a small positive value.
        let sample = |frame: usize, ch: usize| {
            let o = MSG + 48 + (frame * 48 + ch) * 4;
            f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
        };
        assert_eq!(sample(0, 0), 0.0);
        assert!(sample(1, 0) > 0.0 && sample(1, 0) < 1e-3);
        // Channel 47 is 25 kHz: the second frame is larger by far.
        assert!(sample(1, 47) > sample(1, 0));
    }

    #[test]
    fn control_message_is_the_pocs_span() {
        // control.rs span: first tick at pos / 4, one past the last sample.
        assert_eq!(control_ticks(48), 13);
        let mut b = vec![0u8; MSG + 53];
        assert_eq!(encode_control_msg(&mut b, 48, 96_000), MSG + 53);
        assert_eq!(b[0] >> 4, K_CONTROL);
        assert_eq!(b[1], 1);
        assert_eq!(&b[2..4], &13u16.to_le_bytes()); // ticks
        assert_eq!(&b[4..6], &53u16.to_le_bytes()); // payload length
        assert_eq!(b[MSG], FLOOD_CONTROLLER);
    }

    #[test]
    fn datagram_length() {
        // 88-byte header + audio message (8 + 9 264) + controller message
        // (8 + 53): 9 421 bytes, the POC's steady-state feed datagram.
        assert_eq!(datagram_len(48, 48), 88 + 8 + 9264 + 8 + 53);
        assert_eq!(datagram_len(48, 48), 9421);
        let mut b = vec![0u8; datagram_len(48, 48)];
        assert_eq!(encode_body(&mut b, 48, 48, 96_000), 9421);
    }
}
