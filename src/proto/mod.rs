use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub mod http;
pub mod icmp;
pub mod ip;
pub mod tcp;

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
            7 => Self::UDP,
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
            Protocol::UDP => 7,
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
