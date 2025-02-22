use crate::proto::{
    icmp::Icmp,
    ip::{Ip, IpHeaderWriter},
    tcp::Tcp,
    NetworkBuffer, Protocol,
};

use super::{icmp::IcmpHandler, tcp::TcpHandler, Handler};

pub struct IpHandler {
    pub icmp: IcmpHandler,
    pub tcp: TcpHandler,
}

impl Handler<Ip<'_>> for IpHandler {
    type Retrun = NetworkBuffer;

    fn handle(&mut self, ip_header: Ip) -> anyhow::Result<Self::Retrun> {
        println!("IpHeader: {}", ip_header);

        let ttl = ip_header.ttl();
        let src = ip_header.source();
        let dest = ip_header.destination();
        let protocol = ip_header.protocol();

        let mut inner = if ip_header.protocol() == Protocol::TCP {
            let tcp = Tcp::parse(ip_header)?;
            self.tcp.handle(tcp)?
        } else if ip_header.protocol() == Protocol::ICMP {
            let icmp = Icmp::parse(ip_header)?;
            self.icmp.handle(icmp)?;
            NetworkBuffer::empty()
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
