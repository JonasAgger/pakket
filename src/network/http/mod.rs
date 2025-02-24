use crate::proto::{NetworkBuffer, ProtocolBuffer, http::HttpReq, tcp::Tcp};

use super::Handler;

pub struct HttpHandler {}

const RESPONSE: &[u8; 35] = b"HTTP/1.1 200\r\nContent-Length: 0\r\n\r\n";

impl<P: ProtocolBuffer> Handler<Tcp<P>> for HttpHandler {
    type Retrun = (NetworkBuffer, Tcp<P>);
    fn handle(&mut self, msg: Tcp<P>) -> anyhow::Result<Self::Retrun> {
        let http = HttpReq::parse(msg);

        tracing::info!("{}", http);

        let mut buf = NetworkBuffer::new(35);

        buf.extend_from_slice(RESPONSE);

        Ok((buf, http.into_inner()))
    }
}
