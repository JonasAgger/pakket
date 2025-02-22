use std::fmt::Display;

use super::ProtocolBuffer;

pub struct Http<P: ProtocolBuffer> {
    inner: P,
}

impl<P: ProtocolBuffer> Display for Http<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Http")?;

        writeln!(f, "{}", self.inner)
    }
}
