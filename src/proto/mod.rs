use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub mod http;
pub mod icmp;
pub mod ip;
pub mod tcp;
pub mod udp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    ICMP,
    GatewayToGateway,
    TCP,
    UDP,
    Unknown(u8),
}

impl From<u8> for Protocol {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::ICMP,
            3 => Self::GatewayToGateway,
            6 => Self::TCP,
            17 => Self::UDP,
            other => Self::Unknown(other),
        }
    }
}

impl From<Protocol> for u8 {
    fn from(value: Protocol) -> Self {
        match value {
            Protocol::ICMP => 1,
            Protocol::GatewayToGateway => 3,
            Protocol::TCP => 6,
            Protocol::UDP => 17,
            Protocol::Unknown(val) => val,
        }
    }
}

pub trait ProtocolBuffer: Display {
    fn buf(&self) -> &[u8];
}

pub struct NetworkBuffer(Vec<u8>);

impl NetworkBuffer {
    pub fn new(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn new_zeroed(capacity: usize) -> Self {
        Self(vec![0; capacity])
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn extend(&mut self, other: NetworkBuffer) {
        self.0.extend(other.0);
    }
}

impl<T: AsRef<[u8]>> From<T> for NetworkBuffer {
    fn from(value: T) -> Self {
        let data = value.as_ref();
        let mut nb = NetworkBuffer::new(data.len());
        nb.extend_from_slice(data);
        nb
    }
}

impl ProtocolBuffer for NetworkBuffer {
    fn buf(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for NetworkBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NetworkBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for NetworkBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Buf")
    }
}
