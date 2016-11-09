extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{UdpSocketImpl, UdpConnection, UdpConnectionEvent};

fn main() {
    use crufty::net::udp::MockUdpSocket;

    let mut socket = MockUdpSocket::bind(&"127.0.0.1:12365".parse().unwrap()).unwrap();

    socket.latency = Duration::from_millis(250);
    socket.jitter = Duration::from_millis(30);
    socket.packet_loss_ratio = 0.1;

    let mut conn = UdpConnection::from_socket(socket,
                                              &"127.0.0.1:12366".parse().unwrap(),
                                              Duration::from_secs(3));

    // interval at which we send messages
    let send_interval = Duration::from_secs(1);

    let mut event_buffer = Vec::new();

    loop {
        let msg = "Hello, Udp!".as_bytes();
        conn.send_bytes(&msg);
        // println!("msg: {:?}", msg);

        conn.update(Instant::now() + send_interval, &mut event_buffer);

        // println!("{:?}", event_buffer);
        for event in event_buffer.drain(..) {
            match event {
                mto @ UdpConnectionEvent::MessageTimedOut(_) => println!("{:?}", mto),
                _ => {}
            }
        }
    }
}
