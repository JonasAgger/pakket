use serde::de::DeserializeOwned;

use crate::proto::{
    ProtocolBuffer,
    http::{HttpReq, HttpResp},
};

pub struct ReqBody<T: DeserializeOwned>(pub T);
pub struct ReqPath(pub String);

pub trait FromHttpRequest<P: ProtocolBuffer> {
    fn from_context(context: &HttpReq<P>) -> Self;
}

impl<P: ProtocolBuffer> FromHttpRequest<P> for ReqPath {
    fn from_context(context: &HttpReq<P>) -> Self {
        ReqPath(context.path().to_owned())
    }
}

impl<T: DeserializeOwned, P: ProtocolBuffer> FromHttpRequest<P> for ReqBody<T> {
    fn from_context(context: &HttpReq<P>) -> Self {
        let data = serde_json::from_str(context.data()).unwrap();
        ReqBody(data)
    }
}

pub trait Handler<T, P: ProtocolBuffer> {
    fn call(self, context: &HttpReq<P>) -> HttpResp;
}

impl<F, T, P> Handler<T, P> for F
where
    F: Fn(T) -> HttpResp,
    T: FromHttpRequest<P>,
    P: ProtocolBuffer,
{
    fn call(self, context: &HttpReq<P>) -> HttpResp {
        (self)(T::from_context(context))
    }
}

// impl<T1, T2, F, P> Handler<(T1, T2), P> for F
// where
//     F: Fn(T1, T2) -> HttpResp,
//     T1: FromHttpRequest<P>,
//     T2: FromHttpRequest<P>,
//     P: ProtocolBuffer,
// {
//     fn call(self, context: &HttpReq<P>) {
//         (self)(T1::from_context(&context), T2::from_context(&context));
//     }
// }

pub fn trigger<T, H, P>(context: &HttpReq<P>, handler: H) -> HttpResp
where
    H: Handler<T, P>,
    P: ProtocolBuffer,
{
    handler.call(context)
}
