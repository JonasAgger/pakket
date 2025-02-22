mod connections;
mod state;

use anyhow::bail;
use connections::TcpConnections;

use crate::proto::{
    ip::Ip,
    tcp::{Tcp, TcpControl, TcpHeaderWriter},
    NetworkBuffer,
};

use super::{http::HttpHandler, Handler};

pub struct TcpHandler {
    listen_port: u16,
    connections: TcpConnections,
    higher_level_handler: HttpHandler,
}

impl TcpHandler {
    pub fn new(port: u16, handler: HttpHandler) -> Self {
        Self {
            listen_port: port,
            connections: Default::default(),
            higher_level_handler: handler,
        }
    }
}

impl Handler<Tcp<Ip<'_>>> for TcpHandler {
    type Retrun = NetworkBuffer;
    fn handle(&mut self, tcp_header: Tcp<Ip>) -> anyhow::Result<Self::Retrun> {
        println!("TcpHeader: {}", tcp_header);

        if tcp_header.destination_port() != self.listen_port {
            bail!(
                "Received TCP message to wrong port! Expected: {} -> Actual: {}",
                self.listen_port,
                tcp_header.destination_port()
            )
        }

        let connection = self.connections.get(&tcp_header);

        let (msg, writer) = match connection.handle(tcp_header)? {
            state::TcpControlMessage::None(tcp, writer) => (tcp, writer),
            state::TcpControlMessage::Intercepted(tcp_control_message) => {
                return Ok(tcp_control_message)
            }
        };

        let (buf, msg) = self.higher_level_handler.handle(msg)?;

        let msg = if !buf.is_empty() || msg.control().contains(TcpControl::PSH) {
            writer.data(buf).calc_checksum(msg.inner()).to_buf()
        } else {
            buf
        };

        Ok(msg)
    }
}
