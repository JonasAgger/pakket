use crate::proto::{ProtocolBuffer, icmp::Icmp};

use super::Handler;

pub struct IcmpHandler;

impl<P: ProtocolBuffer> Handler<Icmp<P>> for IcmpHandler {
    type Retrun = ();

    fn handle(&mut self, icmp_msg: Icmp<P>) -> anyhow::Result<Self::Retrun> {
        tracing::info!("Icmp: {}", icmp_msg);

        Ok(())
    }
}
