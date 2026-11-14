//! Command-line configuration for the sender (AC 1, AC 2).

use std::net::SocketAddr;

use crate::payload::DEFAULT_UDP_PAYLOAD_LEN;

/// 2 000 packets/s, one stream at the default slice (C-02).
pub const DEFAULT_RATE_PPS: u64 = 2_000;

/// Ticket 001-01: at least 100 000 packets per stamp-distribution run.
pub const DEFAULT_COUNT: u64 = 100_000;

/// Loopback is the safe default target for a probe.
pub const DEFAULT_DEST: &str = "127.0.0.1:9000";

/// Everything one sender run needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerConfig {
    /// Destination IPv4 address and UDP port (C-100: IPv4 only in v1).
    pub dest: SocketAddr,
    /// Configured packets per second.
    pub rate_pps: u64,
    /// Configured number of datagrams to send.
    pub count: u64,
    /// UDP payload length in bytes, 1 252 by default (C-137).
    pub udp_payload_len: usize,
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            dest: DEFAULT_DEST.parse().expect("default dest is a valid SocketAddr"),
            rate_pps: DEFAULT_RATE_PPS,
            count: DEFAULT_COUNT,
            udp_payload_len: DEFAULT_UDP_PAYLOAD_LEN,
        }
    }
}

impl PeerConfig {
    /// Parse `<dest> [--rate <pps>] [--count <n>] [--payload-len <bytes>]`.
    pub fn from_args<I, S>(args: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        use crate::payload::HEADER_LEN;

        let mut config = PeerConfig {
            dest: "0.0.0.0:0".parse().expect("placeholder socket address"),
            rate_pps: DEFAULT_RATE_PPS,
            count: DEFAULT_COUNT,
            udp_payload_len: DEFAULT_UDP_PAYLOAD_LEN,
        };
        let mut dest: Option<SocketAddr> = None;

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let arg = arg.as_ref();
            match arg {
                "--rate" => {
                    let value = args.next().ok_or(ConfigError::MissingValue { flag: "--rate" })?;
                    let value = value.as_ref();
                    config.rate_pps = value
                        .parse()
                        .map_err(|_| ConfigError::InvalidValue {
                            flag: "--rate",
                            value: value.to_string(),
                        })?;
                }
                "--count" => {
                    let value = args.next().ok_or(ConfigError::MissingValue { flag: "--count" })?;
                    let value = value.as_ref();
                    config.count = value
                        .parse()
                        .map_err(|_| ConfigError::InvalidValue {
                            flag: "--count",
                            value: value.to_string(),
                        })?;
                }
                "--payload-len" => {
                    let value = args.next().ok_or(ConfigError::MissingValue { flag: "--payload-len" })?;
                    let value = value.as_ref();
                    config.udp_payload_len = value
                        .parse()
                        .map_err(|_| ConfigError::InvalidValue {
                            flag: "--payload-len",
                            value: value.to_string(),
                        })?;
                }
                _ if arg.starts_with("--") => return Err(ConfigError::UnknownFlag(arg.to_string())),
                _ if dest.is_none() => {
                    let addr: SocketAddr = arg
                        .parse()
                        .map_err(|_| ConfigError::InvalidValue {
                            flag: "dest",
                            value: arg.to_string(),
                        })?;
                    if !addr.is_ipv4() {
                        // C-100: IPv4 only in v1.
                        return Err(ConfigError::InvalidValue {
                            flag: "dest",
                            value: arg.to_string(),
                        });
                    }
                    dest = Some(addr);
                }
                _ => {
                    return Err(ConfigError::InvalidValue {
                        flag: "dest",
                        value: arg.to_string(),
                    })
                }
            }
        }

        config.dest = dest.ok_or(ConfigError::MissingDestination)?;
        if config.rate_pps == 0 {
            return Err(ConfigError::RateZero);
        }
        if config.count == 0 {
            return Err(ConfigError::CountZero);
        }
        if config.udp_payload_len < HEADER_LEN {
            return Err(ConfigError::PayloadTooShort {
                len: config.udp_payload_len,
                min: HEADER_LEN,
            });
        }
        Ok(config)
    }
}

/// Why a command line is not usable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// No destination address was given.
    MissingDestination,
    /// A flag was not recognised.
    UnknownFlag(String),
    /// A flag that takes a value was given none.
    MissingValue { flag: &'static str },
    /// A flag's value is not a number.
    InvalidValue { flag: &'static str, value: String },
    /// The rate must be at least 1 packet/s.
    RateZero,
    /// The count must be at least 1.
    CountZero,
    /// The payload cannot be shorter than the sequence + time header.
    PayloadTooShort { len: usize, min: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::{DEFAULT_UDP_PAYLOAD_LEN, HEADER_LEN};

    #[test]
    fn defaults_match_the_ticket() {
        // AC 1: a run with defaults sends exactly 100 000 datagrams at
        // 2 000 packets/s. AC 2: UDP payload 1 252 bytes.
        let config = PeerConfig::default();
        assert_eq!(config.count, 100_000);
        assert_eq!(config.rate_pps, 2_000);
        assert_eq!(config.udp_payload_len, DEFAULT_UDP_PAYLOAD_LEN);
        assert_eq!(config.udp_payload_len, 1_252);
        assert!(config.dest.ip().is_loopback(), "default target is loopback");
        assert_ne!(config.dest.port(), 0);
    }

    #[test]
    fn command_line_overrides_rate_count_and_payload_length() {
        let config = PeerConfig::from_args([
            "127.0.0.1:9999",
            "--rate",
            "1000",
            "--count",
            "25",
            "--payload-len",
            "512",
        ])
        .expect("valid command line");
        assert_eq!(config.dest, "127.0.0.1:9999".parse().unwrap());
        assert_eq!(config.rate_pps, 1_000);
        assert_eq!(config.count, 25);
        assert_eq!(config.udp_payload_len, 512);
    }

    #[test]
    fn command_line_rejects_a_zero_rate() {
        assert_eq!(
            PeerConfig::from_args(["127.0.0.1:9000", "--rate", "0"]),
            Err(ConfigError::RateZero)
        );
    }

    #[test]
    fn command_line_rejects_a_zero_count() {
        assert_eq!(
            PeerConfig::from_args(["127.0.0.1:9000", "--count", "0"]),
            Err(ConfigError::CountZero)
        );
    }

    #[test]
    fn command_line_rejects_a_payload_shorter_than_the_header() {
        assert_eq!(
            PeerConfig::from_args(["127.0.0.1:9000", "--payload-len", "8"]),
            Err(ConfigError::PayloadTooShort {
                len: 8,
                min: HEADER_LEN,
            })
        );
    }

    #[test]
    fn command_line_requires_a_destination() {
        assert_eq!(
            PeerConfig::from_args(["--rate", "2000"]),
            Err(ConfigError::MissingDestination)
        );
    }
}
