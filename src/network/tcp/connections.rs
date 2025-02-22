use std::collections::HashMap;

use crate::proto::{ip::Ip, tcp::Tcp};

use super::state::TcpState;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Quad(u32, u16);

impl Tcp<Ip<'_>> {
    fn quad(&self) -> Quad {
        Quad(self.inner().source(), self.source_port())
    }
}

#[derive(Debug, Default)]
pub struct TcpConnections {
    inner: HashMap<Quad, TcpState>,
}

impl TcpConnections {
    pub fn get(&mut self, msg: &Tcp<Ip<'_>>) -> &mut TcpState {
        self.inner.entry(msg.quad()).or_default()
    }
}
