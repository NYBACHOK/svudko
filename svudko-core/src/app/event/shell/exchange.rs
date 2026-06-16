use std::net::IpAddr;

use crux_core::capability::Operation;

use crate::{event::ExchangeEvent, resolvers::exchange::ExchangeErrors};

#[derive(Clone, Debug)]
pub enum ExchangeRequest {
    Connect((IpAddr, String)),
}

impl Operation for ExchangeRequest {
    type Output = Result<ExchangeEvent, ExchangeErrors>;
}
