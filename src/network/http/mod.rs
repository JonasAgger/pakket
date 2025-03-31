mod utils;

pub use utils::*;

use crate::{
    application,
    proto::{NetworkBuffer, ProtocolBuffer, http::HttpReq, tcp::Tcp},
};

use super::Handler;

pub struct HttpHandler {
    server: Option<application::Api>,
}

impl HttpHandler {
    pub fn new(h: application::Api) -> Self {
        Self { server: Some(h) }
    }

    pub fn none() -> Self {
        Self { server: None }
    }
}

const RESPONSE: &[u8; 19] = b"HTTP/1.1 404 OK\r\n\r\n";

impl<P: ProtocolBuffer> Handler<Tcp<P>> for HttpHandler {
    type ReturnType = (NetworkBuffer, Tcp<P>);
    fn handle(&mut self, msg: Tcp<P>) -> anyhow::Result<Self::ReturnType> {
        let http = HttpReq::parse(msg);

        tracing::info!("{}", http);

        let buf = if let Some(server) = self.server.as_mut() {
            let resp = server.on_request(&http);
            resp.to_buf()
        } else {
            let mut buf = NetworkBuffer::new(RESPONSE.len());

            buf.extend_from_slice(RESPONSE);

            buf
        };

        Ok((buf, http.into_inner()))
    }
}
