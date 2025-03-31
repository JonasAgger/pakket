mod connections;
mod state;

use std::sync::Arc;

use anyhow::bail;
use connections::TcpConnections;

use crate::proto::{
    NetworkBuffer,
    ip::Ip,
    tcp::{Tcp, TcpControl},
};

use super::{Handler, http::HttpHandler};

pub struct TcpHandler {
    listen_port: u16,
    connections: TcpConnections,
    higher_level_handler: HttpHandler,
}

impl TcpHandler {
    pub fn new(port: u16, handler: HttpHandler, nic: Arc<tun::Device>) -> Self {
        Self {
            listen_port: port,
            connections: TcpConnections::new(nic),
            higher_level_handler: handler,
        }
    }
}

impl Handler<Ip<'_>> for TcpHandler {
    type ReturnType = NetworkBuffer;
    fn handle(&mut self, ip: Ip) -> anyhow::Result<Self::ReturnType> {
        let tcp_header = Tcp::parse(ip)?;
        tracing::info!("TcpHeader: {}", tcp_header);

        if tcp_header.destination_port() != self.listen_port {
            bail!(
                "Received TCP message to wrong port! Expected: {} -> Actual: {}",
                self.listen_port,
                tcp_header.destination_port()
            )
        }

        let (connection, quad) = self.connections.get(&tcp_header);

        let msg = match connection.handle(tcp_header)? {
            state::TcpControlMessage::None(tcp) => tcp,
            state::TcpControlMessage::Intercepted(tcp_control_message) => {
                return Ok(tcp_control_message);
            }
            state::TcpControlMessage::Closed => {
                self.connections.remove(quad);
                return Ok(NetworkBuffer::empty());
            }
        };

        let (buf, msg) = if msg.control().contains(TcpControl::PSH) {
            self.higher_level_handler.handle(msg)?
        } else {
            (NetworkBuffer::empty(), msg)
        };

        Ok(connection.send(buf, msg))
    }
}
