use std::net::IpAddr;

use svudko_common::Operation;

use crate::event::ExchangeEvent;

#[derive(Clone, Debug)]
pub enum ExchangeCoreRequest {
    Connect((IpAddr, String)),
}

impl Operation for ExchangeCoreRequest {
    type Output = ExchangeEvent;
}
