use std::fmt::Display;

use super::ProtocolBuffer;
use anyhow::Result;

pub struct Icmp<P: ProtocolBuffer> {
    inner: P,
}

impl<P: ProtocolBuffer> ProtocolBuffer for Icmp<P> {
    fn buf(&self) -> &[u8] {
        self.inner.buf()
    }
}

impl<P: ProtocolBuffer> Icmp<P> {
    pub fn parse(proto: P) -> Result<Self> {
        Ok(Self { inner: proto })
    }

    pub fn icmp_type(&self) -> u8 {
        self.inner.buf()[0]
    }

    pub fn icmp_code(&self) -> u8 {
        self.inner.buf()[1]
    }
}

impl<P: ProtocolBuffer> Display for Icmp<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ICMP")?;
        writeln!(f, "- Type: {}", self.icmp_type())?;
        writeln!(f, "- Code: {}", self.icmp_code())?;
        writeln!(f, "- Remainder: {:?}", &self.inner.buf()[2..])?;

        writeln!(f, "{}", self.inner)
    }
}
