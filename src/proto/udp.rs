use std::fmt::Display;

use crate::utils;

use super::{NetworkBuffer, ProtocolBuffer};

impl<P: ProtocolBuffer> ProtocolBuffer for Udp<P> {
    fn buf(&self) -> &[u8] {
        &self.inner.buf()[8..]
    }
}

pub struct Udp<P: ProtocolBuffer> {
    inner: P,
}

impl<P: ProtocolBuffer> Udp<P> {
    pub fn parse(proto: P) -> anyhow::Result<Self> {
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

    pub fn length(&self) -> u16 {
        utils::read_u16(&self.inner.buf()[4..])
    }

    pub fn checksum(&self) -> u16 {
        utils::read_u16(&self.inner.buf()[6..])
    }
}

pub struct UdpHeaderWriter {
    buf: NetworkBuffer,
}

impl UdpHeaderWriter {
    const DEFAULT_SIZE: u16 = 8;

    pub fn new(source_port: u16, destination_port: u16) -> Self {
        let mut buf = NetworkBuffer::new(Self::DEFAULT_SIZE as usize);
        buf.extend_from_slice(&source_port.to_be_bytes());
        buf.extend_from_slice(&destination_port.to_be_bytes());
        buf.extend_from_slice(&Self::DEFAULT_SIZE.to_be_bytes());
        buf.resize(Self::DEFAULT_SIZE as usize, 0);

        Self { buf }
    }

    pub fn data(mut self, data: NetworkBuffer) -> Self {
        if !data.is_empty() {
            self.buf.extend(data);
            let len = self.buf.len() as u16;
            self.buf[4..6].copy_from_slice(&len.to_be_bytes());
            self
        } else {
            self
        }
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

        let checksum =
            utils::ones_complement_with_no_zero(utils::add_slice(ip_header_sum, &self.buf)).to_be();
        self.buf[16..18].copy_from_slice(&checksum.to_be_bytes());

        self
    }

    pub fn to_buf(self) -> NetworkBuffer {
        self.buf
    }
}

impl<P: ProtocolBuffer> Display for Udp<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "UDP")?;
        writeln!(f, "- Src: {}", self.source_port())?;
        writeln!(f, "- Dest: {}", self.destination_port())?;
        writeln!(f, "- Len: {}", self.length())
    }
}
