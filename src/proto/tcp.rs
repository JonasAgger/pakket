use std::fmt::Display;

use crate::utils;

use super::{NetworkBuffer, ProtocolBuffer};
use anyhow::Result;

pub struct Tcp<P: ProtocolBuffer> {
    inner: P,
}

impl<P: ProtocolBuffer> ProtocolBuffer for Tcp<P> {
    fn buf(&self) -> &[u8] {
        let buf = self.inner.buf();
        &buf[self.header_length()..]
    }
}

impl<P: ProtocolBuffer> Tcp<P> {
    pub fn parse(proto: P) -> Result<Self> {
        Ok(Self { inner: proto })
    }

    pub fn inner(&self) -> &P {
        &self.inner
    }

    pub fn source_port(&self) -> u16 {
        utils::read_u16(self.inner.buf())
    }

    pub fn destination_port(&self) -> u16 {
        utils::read_u16(&self.inner.buf()[2..])
    }

    pub fn sequence_number(&self) -> u32 {
        utils::read_u32(&self.inner.buf()[4..])
    }

    pub fn ack_number(&self) -> u32 {
        utils::read_u32(&self.inner.buf()[8..])
    }

    pub fn header_length(&self) -> usize {
        let len = unsafe { *self.inner.buf().get_unchecked(12) >> 4 };
        // Length is represented as 32 bit words, so 4 bytes
        (len * 4) as usize
    }

    pub fn control(&self) -> TcpControl {
        TcpControl::from_bits_retain(unsafe { *self.inner.buf().get_unchecked(13) })
    }

    pub fn window(&self) -> u16 {
        utils::read_u16(&self.inner.buf()[14..])
    }

    pub fn urgent_pointer(&self) -> u16 {
        utils::read_u16(&self.inner.buf()[18..])
    }
}

pub struct TcpHeaderWriter {
    buf: NetworkBuffer,
}

impl TcpHeaderWriter {
    const DEFAULT_SIZE: u8 = 5 << 4; // 5 * 4

    pub fn new(
        source_port: u16,
        destination_port: u16,
        sequence_number: u32,
        ack_number: u32,
    ) -> Self {
        let mut buf = NetworkBuffer::new(20);
        buf.extend_from_slice(&source_port.to_be_bytes());
        buf.extend_from_slice(&destination_port.to_be_bytes());
        buf.extend_from_slice(&sequence_number.to_be_bytes());
        buf.extend_from_slice(&ack_number.to_be_bytes()); // Ack Number
        buf.push(Self::DEFAULT_SIZE); // 4bit total header len in 4 byte sizes. eg. 5 == 20 bytes.
        buf.resize(20, 0);
        buf[14..16].copy_from_slice(&1024u16.to_be_bytes());

        Self { buf }
    }

    pub fn data(mut self, data: NetworkBuffer) -> Self {
        if !data.is_empty() {
            self.buf.extend(data);
            self.set(TcpControl::PSH)
        } else {
            self
        }
    }

    pub fn set(mut self, control: TcpControl) -> Self {
        let current = TcpControl::from_bits_retain(unsafe { *self.buf.get_unchecked(13) });
        self.buf[13] = current.union(control).bits();
        self
    }

    pub fn calc_checksum(mut self, ip_header: &super::ip::Ip<'_>) -> Self {
        let ip_header_sum = {
            let length = self.buf.len() as u16;
            let mut sum = 0;
            sum = utils::add_4bytes(sum, ip_header.destination2());
            sum = utils::add_4bytes(sum, ip_header.source2());
            sum = utils::add_2bytes(sum, [0, ip_header.protocol().into()]);
            sum = utils::add_2bytes(sum, length.to_be_bytes());
            sum
        };

        let checksum = utils::ones_complement(utils::add_slice(ip_header_sum, &self.buf)).to_be();
        self.buf[16..18].copy_from_slice(&checksum.to_be_bytes());

        self
    }

    pub fn to_buf(self) -> NetworkBuffer {
        self.buf
    }
}

impl<P: ProtocolBuffer> Display for Tcp<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TCP")?;
        writeln!(f, "- Src: {}", self.source_port())?;
        writeln!(f, "- Dest: {}", self.destination_port())?;
        writeln!(f, "- Seq: {}", self.sequence_number())?;
        writeln!(f, "- Ack: {}", self.ack_number())?;
        writeln!(f, "- Len: {}", self.header_length())?;
        writeln!(f, "- Control: {:?}", self.control())?;
        writeln!(f, "- Window: {:?}", self.window())?;
        writeln!(f, "- UrgentPointer: {:?}", self.urgent_pointer())
    }
}

use bitflags::bitflags;
bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TcpControl: u8 {
        const FIN = 0b0000_0001;
        const SYN = 0b0000_0010;
        const RST = 0b0000_0100;
        const PSH = 0b0000_1000;
        const ACK = 0b0001_0000;
        const URG = 0b0010_0000;
        const ECE = 0b0100_0000;
        const CWR = 0b1000_0000;
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::ip::{Ip, IpHeaderWriter};

    use super::*;

    #[test]
    fn can_serialize_tcp_header() {
        const S_ADDR: [u8; 4] = [10, 100, 0, 5];
        const D_ADDR: [u8; 4] = [10, 100, 0, 10];
        const SP: u16 = 3000;
        const DP: u16 = 3001;
        const SN: u32 = 5;
        const AN: u32 = 10;

        let data = NetworkBuffer::empty();
        let ip_buf = IpHeaderWriter::new(
            u32::from_be_bytes(S_ADDR),
            u32::from_be_bytes(D_ADDR),
            crate::proto::Protocol::TCP,
            64,
            NetworkBuffer::empty(),
        )
        .to_buf();
        let ip = Ip::parse(&ip_buf).unwrap();

        let tcp = TcpHeaderWriter::new(SP, DP, SN, AN)
            .set(TcpControl::ACK)
            .calc_checksum(&ip)
            .to_buf();

        let mut tcp2 = etherparse::TcpHeader::new(SP, DP, SN, 1024);
        tcp2.acknowledgment_number = AN;
        tcp2.ack = true;
        tcp2.checksum = tcp2
            .calc_checksum_ipv4(
                &etherparse::Ipv4Header::new(0, 64, etherparse::IpNumber::TCP, S_ADDR, D_ADDR)
                    .unwrap(),
                &[],
            )
            .unwrap();

        let mut buf = Vec::new();
        tcp2.write(&mut buf).unwrap();

        assert_eq!(tcp.as_slice(), buf.as_slice())
    }
}
