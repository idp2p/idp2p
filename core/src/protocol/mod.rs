use std::iter;

use libp2p::request_response::{ProtocolSupport, RequestResponse};

use self::codec::{IdCodec, IdProtocol};

pub mod codec;

pub fn build_request_response() -> RequestResponse<IdCodec> {
    let req_res = RequestResponse::new(
        IdCodec(),
        iter::once((IdProtocol(), ProtocolSupport::Full)),
        Default::default(),
    );
    req_res
}
