use crate::proto::{
    NetworkBuffer, Protocol,
    ip::{Ip, IpHeaderWriter},
};

use super::{Handler, icmp::IcmpHandler, tcp::TcpHandler, udp::UdpHandler};

pub struct IpHandler {
    pub icmp: IcmpHandler,
    pub udp: UdpHandler,
    pub tcp: TcpHandler,
}

impl Handler<Ip<'_>> for IpHandler {
    type ReturnType = NetworkBuffer;

    fn handle(&mut self, ip_header: Ip) -> anyhow::Result<Self::ReturnType> {
        tracing::info!("IpHeader: {}", ip_header);

        let ttl = ip_header.ttl();
        let src = ip_header.source();
        let dest = ip_header.destination();
        let protocol = ip_header.protocol();

        let mut inner = if ip_header.protocol() == Protocol::TCP {
            self.tcp.handle(ip_header)?
        } else if ip_header.protocol() == Protocol::UDP {
            self.udp.handle(ip_header)?
        } else if ip_header.protocol() == Protocol::ICMP {
            self.icmp.handle(ip_header)?
        } else {
            NetworkBuffer::empty()
        };

        if !inner.is_empty() {
            let writer = IpHeaderWriter::new(dest, src, protocol, ttl, inner);
            inner = writer.to_buf();
        }

        Ok(inner)
    }
}
