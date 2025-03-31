use crate::proto::ProtocolBuffer;

pub mod http;
pub mod icmp;
pub mod ip;
pub mod tcp;
pub mod udp;

pub trait Handler<P: ProtocolBuffer> {
    type ReturnType;
    fn handle(&mut self, msg: P) -> anyhow::Result<Self::ReturnType>;
}
