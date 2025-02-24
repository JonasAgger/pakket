use std::{collections::HashMap, sync::Arc};

use crate::proto::{ip::Ip, tcp::Tcp};

use super::state::TcpState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Quad(u32, u16);

impl Tcp<Ip<'_>> {
    fn quad(&self) -> Quad {
        Quad(self.inner().source(), self.source_port())
    }
}

pub struct TcpConnections {
    inner: HashMap<Quad, TcpState>,
    nic: Arc<tun::Device>,
}

impl TcpConnections {
    pub fn new(nic: Arc<tun::Device>) -> Self {
        Self {
            inner: HashMap::new(),
            nic,
        }
    }

    pub fn get(&mut self, msg: &Tcp<Ip<'_>>) -> (&mut TcpState, Quad) {
        let quad = msg.quad();
        let state = self
            .inner
            .entry(quad)
            .or_insert_with(|| TcpState::new(self.nic.clone()));
        (state, quad)
    }

    pub fn remove(&mut self, quad: Quad) {
        self.inner.remove(&quad);
    }
}
