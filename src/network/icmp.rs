use crate::proto::{icmp::Icmp, ProtocolBuffer};

use super::Handler;

pub struct IcmpHandler;

impl<P: ProtocolBuffer> Handler<Icmp<P>> for IcmpHandler {
    type Retrun = ();

    fn handle(&mut self, icmp_msg: Icmp<P>) -> anyhow::Result<Self::Retrun> {
        println!("Icmp: {}", icmp_msg);

        Ok(())
    }
}
