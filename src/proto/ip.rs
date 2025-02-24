use std::fmt::Display;

use crate::utils;

use super::{NetworkBuffer, Protocol, ProtocolBuffer};

use anyhow::Result;

const IP_HEADER_LEN_MIN: usize = 20;

impl<'a> ProtocolBuffer for Ip<'a> {
    fn buf(&self) -> &[u8] {
        &self.remainder()
    }
}

pub struct Ip<'a> {
    data: &'a [u8],
}

impl<'a> Ip<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        if bytes.len() < IP_HEADER_LEN_MIN {
            anyhow::bail!("Expected at least 20 bytes")
        }

        let s = Self { data: bytes };

        if s.header_length() > s.data.len() {
            anyhow::bail!(
                "Bytes recv less than header length: HeaderLen: {}, DataLen: {}",
                s.header_length(),
                s.data.len()
            )
        }

        Ok(s)
    }
}

impl<'a> Ip<'a> {
    pub fn header_length(&self) -> usize {
        let len = unsafe { *self.data.get_unchecked(0) & 0xf };
        // Length is represented as 32 bit words, so 4 bytes
        (len * 4) as usize
    }

    pub fn remainder(&self) -> &'a [u8] {
        &self.data[self.header_length()..]
    }

    pub fn total_length(&self) -> u16 {
        utils::read_u16(&self.data[2..4])
    }

    pub fn ttl(&self) -> u8 {
        self.data[8]
    }

    pub fn protocol(&self) -> Protocol {
        self.data[9].into()
    }

    pub fn checksum(&self) -> u16 {
        utils::read_u16(&self.data[10..12])
    }

    pub fn source(&self) -> u32 {
        utils::read_u32(&self.data[12..])
    }

    pub fn source2(&self) -> [u8; 4] {
        self.data[12..16].try_into().unwrap()
    }

    pub fn destination(&self) -> u32 {
        utils::read_u32(&self.data[16..])
    }

    pub fn destination2(&self) -> [u8; 4] {
        self.data[16..20].try_into().unwrap()
    }
}

pub struct IpHeaderWriter {
    buf: NetworkBuffer,
}

impl IpHeaderWriter {
    const VERSION_AND_LEN: u8 = 0b01000101;
    pub fn new(
        source: u32,
        destination: u32,
        protocol: Protocol,
        time_to_live: u8,
        data: NetworkBuffer,
    ) -> Self {
        let mut buf = NetworkBuffer::new_zeroed(20 + data.len());
        buf[0] = Self::VERSION_AND_LEN;
        // buf[1] = 0; // type of service
        buf[2..4].copy_from_slice(&((20 + data.len()) as u16).to_be_bytes());
        // identification buf[4..6]
        // flags
        buf[6] = 0b0100_0000; //dont fragment
                              // fragment
        buf[8] = time_to_live;
        buf[9] = protocol.into();
        buf[12..16].copy_from_slice(&source.to_be_bytes());
        buf[16..20].copy_from_slice(&destination.to_be_bytes());

        let mut s = Self { buf };
        let checksum = s.checksum();
        // Not really sure what endianness we're here tbh.
        s.buf[10..12].copy_from_slice(&checksum.to_ne_bytes());

        // Copy inner data
        s.buf[20..].copy_from_slice(&data);
        s
    }

    pub fn to_buf(self) -> NetworkBuffer {
        self.buf
    }

    pub fn checksum(&self) -> u16 {
        // func returns big endiann, just switch back
        utils::ones_complement(utils::add_slice(0, &self.buf[..20]))
    }
}

impl<'a> Display for Ip<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "IP")?;
        writeln!(f, "- HeaderLen: {}", self.header_length())?;
        writeln!(f, "- TotalLen: {}", self.total_length())?;
        writeln!(f, "- TTL: {}", self.ttl())?;
        writeln!(f, "- Protocol: {:?}", self.protocol())?;
        writeln!(f, "- Source: {:?}", self.source2())?;
        writeln!(f, "- Destination: {:?}", self.destination2())
    }
}
