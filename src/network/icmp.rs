use crate::proto::{NetworkBuffer, ProtocolBuffer, icmp::Icmp};

use super::Handler;

pub struct IcmpHandler;

impl<P: ProtocolBuffer> Handler<P> for IcmpHandler {
    type ReturnType = NetworkBuffer;

    fn handle(&mut self, msg: P) -> anyhow::Result<Self::ReturnType> {
        let icmp_msg = Icmp::parse(msg)?;
        tracing::info!("Icmp: {}", icmp_msg);

        Ok(NetworkBuffer::empty())
    }
}
