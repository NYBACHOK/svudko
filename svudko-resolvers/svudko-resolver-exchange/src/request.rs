use std::net::IpAddr;

use svudko_common::resolver::Operation;

use crate::event::ExchangeEvent;

#[derive(Clone, Debug)]
pub enum ExchangeCoreRequest {
    Connect((IpAddr, String)),
    Send(String)
}

impl Operation for ExchangeCoreRequest {
    type Output = ExchangeEvent;
}
