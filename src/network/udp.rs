use crate::proto::{
    NetworkBuffer, ProtocolBuffer,
    udp::{Udp, UdpHeaderWriter},
};

use super::Handler;

pub struct UdpHandler;

impl<P: ProtocolBuffer> Handler<P> for UdpHandler {
    type ReturnType = NetworkBuffer;

    fn handle(&mut self, msg: P) -> anyhow::Result<Self::ReturnType> {
        let udp_msg = Udp::parse(msg)?;
        tracing::info!("UdpHeader: {}", udp_msg);

        let buf = UdpHeaderWriter::new(udp_msg.destination_port(), udp_msg.source_port())
            .data(udp_msg.buf().into())
            .to_buf();

        Ok(buf)
    }
}
