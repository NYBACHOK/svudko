use std::net::IpAddr;

use crux_core::capability::Operation;

use crate::{event::ExchangeEvent, resolvers::exchange::ExchangeErrors};

#[derive(Clone, Debug)]
pub enum ExchangeCoreRequest {
    Connect((IpAddr, String)),
}

impl Operation for ExchangeCoreRequest {
    type Output = Result<ExchangeEvent, ExchangeErrors>;
}
