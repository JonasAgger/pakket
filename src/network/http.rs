use crate::proto::{tcp::Tcp, NetworkBuffer, ProtocolBuffer};

use super::Handler;

pub struct HttpHandler {}

impl<P: ProtocolBuffer> Handler<Tcp<P>> for HttpHandler {
    type Retrun = (NetworkBuffer, Tcp<P>);
    fn handle(&mut self, msg: Tcp<P>) -> anyhow::Result<Self::Retrun> {
        let data = String::from_utf8_lossy(msg.buf());

        println!("HTTP: {:?}", data);

        Ok((NetworkBuffer::empty(), msg))
    }
}
