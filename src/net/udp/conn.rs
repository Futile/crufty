use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::io::{self, Cursor};
use std::io::prelude::{Write, Read};
use std::collections::VecDeque;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::seqnum::SequenceNumber;
use super::ackstat::AckStatus;

#[derive(Debug)]
struct Buffer(Vec<u8>);

impl Buffer {
    fn new() -> Buffer {
        Buffer(Vec::new())
    }

    fn take(&mut self) -> Vec<u8> {
        ::std::mem::replace(&mut self.0, Vec::new())
    }

    fn done(&mut self, buf: Vec<u8>) {
        if buf.capacity() > self.0.capacity() {
            self.0 = buf;

            // invariant: buffer has to stay cleared
            self.0.clear();
        }
    }
}

#[derive(Debug, Clone)]
struct InFlightInfo {
    seq_num: SequenceNumber,
    sent_time: Instant,
    msg_id: MessageId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MessageId(u64);

const MAGIC_PROTOCOL_ID: u32 = 0xABFECDFE;

type Encoding = BigEndian;

#[derive(Debug, Copy, Clone)]
struct PacketHeader {
    seq_num: SequenceNumber,
    acks: AckStatus,
}

#[derive(Debug)]
pub enum ReceiveEvent<'a> {
    NewAck(MessageId, Duration),
    NewData(&'a [u8]),
}

#[derive(Debug)]
pub struct NewAckEvent {
    pub msg_id: MessageId,
    pub rtt: Duration,
}

pub struct UdpConnection {
    socket: UdpSocket,
    next_local_sequence_number: SequenceNumber,
    ack_control: AckStatus,

    next_message_id: MessageId,
    buffer: Buffer,
    pending_acks: VecDeque<InFlightInfo>,
    packet_timeout_limit: Duration,
}

impl UdpConnection {
    pub fn new(local_addr: &SocketAddr,
               remote_addr: &SocketAddr,
               packet_timeout_limit: Duration)
               -> UdpConnection {
        let socket = UdpSocket::bind(local_addr).unwrap();
        socket.connect(remote_addr).unwrap();

        UdpConnection {
            socket: socket,
            next_local_sequence_number: SequenceNumber::first(),
            ack_control: AckStatus::new(),

            next_message_id: MessageId(0),
            buffer: Buffer::new(),
            pending_acks: VecDeque::new(),
            packet_timeout_limit: packet_timeout_limit,
        }
    }

    /// Changes the duration after which an un-acked packet is considered lost.
    pub fn set_packet_timeout_limit(&mut self, new_timeout: Duration) {
        self.packet_timeout_limit = new_timeout;
    }

    fn write_header<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        // write magic protocol id
        writer.write_u32::<Encoding>(MAGIC_PROTOCOL_ID)?;

        // write sequence number
        self.next_local_sequence_number.write_to_packet::<Encoding, _>(writer)?;

        // write acks
        self.ack_control.write_to_packet::<Encoding, _>(writer)
    }

    fn read_header<R: Read>(reader: &mut R) -> io::Result<PacketHeader> {
        // first is the magic protocol id
        let magic = reader.read_u32::<Encoding>()?;

        // if it doesn't match, return an error
        if magic != MAGIC_PROTOCOL_ID {
            return Err(io::Error::new(io::ErrorKind::Other, "wrong protocol id"));
        }

        // deserialize sequence number and acks
        let seq_num = SequenceNumber::read_from_packet::<Encoding, _>(reader)?;
        let acks = AckStatus::read_from_packet::<Encoding, _>(reader)?;

        // build header and return
        Ok(PacketHeader {
            seq_num: seq_num,
            acks: acks,
        })
    }

    pub fn wrap_payload(&mut self, payload: &[u8]) -> io::Result<(MessageId, Vec<u8>)> {
        // 10 is an overapproximation for our header size, but good enough
        let mut buffer = Vec::with_capacity(payload.len() + 10);

        // write header to buffer
        self.write_header(&mut buffer)?;

        // write msg to buffer
        buffer.write_all(payload)?;

        // create and increase MessageId
        let msg_id = self.next_message_id;
        self.next_message_id.0 += 1;

        // add packet info (seq num, sent-timestamp, message id) to pending acks
        self.pending_acks.push_back(InFlightInfo {
            seq_num: self.next_local_sequence_number,
            sent_time: Instant::now(),
            msg_id: msg_id,
        });

        // increase next sequence number
        self.next_local_sequence_number += 1.into();

        Ok((msg_id, buffer))
    }

    pub fn send_bytes(&mut self, msg: &[u8]) -> io::Result<MessageId> {
        let (msg_id, packet) = self.wrap_payload(msg)?;

        // send packet
        let sent_count = self.socket.send(&packet)?;

        // sanity check, should return an error if we try to send too much
        // (which the unwrap above should catch)
        assert_eq!(sent_count,
                   packet.len(),
                   "only a partial send occured, should not happen??");

        Ok(msg_id)
    }

    pub fn unwrap_payload<'a, F>(&mut self,
                              payload: &'a [u8],
                              mut new_ack_handler: F)
                              -> io::Result<&'a [u8]>
        where F: FnMut(NewAckEvent)
    {
        let mut reader = Cursor::new(payload);

        // read header
        let header = Self::read_header(&mut reader)?;

        // ack the remote packet
        self.ack_control.ack(header.seq_num);

        // only keep un-acked packages in pending acks and update average rtt
        self.pending_acks.retain(|info| {
            if header.acks.is_acked(info.seq_num) {
                // rtt of this packet
                let rtt = info.sent_time.elapsed();

                // give new ack to caller
                new_ack_handler(NewAckEvent{ msg_id: info.msg_id, rtt: rtt });

                // packet was acked, don't keep it in pending queue
                return false;
            }

            // packet not acked yet, keep as pending
            true
        });

        // save until where we've read the buffer
        let reader_pos = reader.position() as usize;

        Ok(&reader.into_inner()[reader_pos..])

    }

    // TODO this should probably return a Result, so we know when e.g. a wrong magic number arrived
    fn receive_packet<F>(&mut self, buffer: &[u8], mut handler: F)
        where F: FnMut(ReceiveEvent)
    {
        let data = self.unwrap_payload(buffer, |new_ack| {
            handler(ReceiveEvent::NewAck(new_ack.msg_id, new_ack.rtt));
        }).unwrap();

        // give rest of the packet to the caller
        handler(ReceiveEvent::NewData(data));

    }

    pub fn recv_with_timeout<F>(&mut self, timeout: Option<Duration>, handler: F)
        where F: FnMut(ReceiveEvent)
    {
        // sanity-check warning
        timeout.map(|to| {
            assert_ne!(to,
                       Duration::new(0, 0),
                       "zero-duration is invalid, use 'None' for blocking")
        });

        // set timeout for the receive
        self.socket.set_read_timeout(timeout).unwrap();

        // get a buffer for receiving
        let mut buffer = self.buffer.take();

        // resize the buffer without allocating, because we want >64kb
        unsafe {
            // see e.g. https://stackoverflow.com/questions/1098897/
            let max_udp_size: usize = 65507;

            // reserve, i.e. allocate, enough memory
            // this is necessary so socket.recv() won't throw away anything..
            buffer.reserve(max_udp_size);

            // this is unsafe
            buffer.set_len(max_udp_size);

            // try receiving a packet (with the timeout set before)
            match self.socket.recv(&mut buffer) {
                // receive successful
                Ok(bytes_read) => {
                    // sanity check
                    assert!(bytes_read <= buffer.capacity());

                    // **IMPORTANT** restrict buffer size to how much is actually valid
                    buffer.set_len(bytes_read);

                    // this is safe now, since we made sure the buffer size is correct
                    self.receive_packet(&buffer, handler);
                }
                Err(e) => {
                    // **IMPORTANT** on any error, set buffer length to 0
                    buffer.set_len(0);

                    // see what error we got
                    match e.kind() {
                        // on timeout do nothing
                        io::ErrorKind::WouldBlock |
                        io::ErrorKind::TimedOut => {}
                        // else, panic
                        _ => panic!(e),
                    }
                }
            };

            // return buffer (this will also clear() it)
            self.buffer.done(buffer);
        }
    }

    pub fn check_for_timeouts<F>(&mut self, mut on_timeout: F)
        where F: FnMut(MessageId)
    {
        let now = Instant::now();

        // borrow checker forces this copy, or I couldn't do it better
        let timeout_limit = self.packet_timeout_limit;

        // retain only not-timeouted packets
        self.pending_acks.retain(|info| {
            // packet is timed out if it has been in flight for more than a certain time
            let timed_out = now.duration_since(info.sent_time) >= timeout_limit;

            // if it timed out, report an event to the caller
            if timed_out {
                on_timeout(info.msg_id);
            }

            // this cleans out packets that have timed out
            !timed_out
        });
    }
}
