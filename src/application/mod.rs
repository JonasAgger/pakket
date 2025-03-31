use serde::Deserialize;

use crate::{
    network::http::{ReqBody, ReqPath, trigger},
    proto::http::HttpResp,
};

pub struct Api;

impl Api {
    pub fn on_request<P: crate::proto::ProtocolBuffer>(
        &mut self,
        request: &crate::proto::http::HttpReq<P>,
    ) -> HttpResp {
        match request.path() {
            "/data" => return trigger(request, on_data),
            "/req" => return trigger(request, on_req),
            path => tracing::info!(?path, "Unmatched path"),
        }

        HttpResp::ok()
    }
}

fn on_data(ReqPath(path): ReqPath) -> HttpResp {
    tracing::info!(?path, "ON DATA!");
    HttpResp::ok()
}

fn on_req(ReqBody(wat): ReqBody<Wat>) -> HttpResp {
    tracing::info!(?wat, "ON REQ!");

    HttpResp::ok()
}

#[derive(Deserialize, Debug)]
struct Wat {
    key1: String,
    key2: String,
}
