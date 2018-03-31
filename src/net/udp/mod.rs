// this is an implementation following http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/

mod seqnum;
mod ackstat;
mod conn;
mod reliability;
mod congcontrol;
mod basic_conn;

pub use self::basic_conn::{BasicUdpConnection};
pub use self::conn::ConnectionWrapper;
pub use self::reliability::{MessageId, ReliabilityWrapper, ReceiveEvent, NewAckEvent};
pub use self::congcontrol::{CongestionControl, CongestionStatus};

use std::time::Duration;

pub fn ns_to_ms(ns: u32) -> u64 {
    ns as u64 / 1_000_000
}

pub fn dur_to_ms(dur: &Duration) -> u64 {
    dur.as_secs() * 1000 + ns_to_ms(dur.subsec_nanos())
}
