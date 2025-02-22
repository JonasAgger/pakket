use crate::proto::ProtocolBuffer;

pub mod http;
pub mod icmp;
pub mod ip;
pub mod tcp;

pub trait Handler<P: ProtocolBuffer> {
    type Retrun;
    fn handle(&mut self, msg: P) -> anyhow::Result<Self::Retrun>;
}
