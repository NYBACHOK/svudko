use std::{collections::HashSet, net::IpAddr};

use svudko_common::{hostname::Hostname, resolver::Operation};

use crate::event::ExchangeEvent;

#[derive(Clone, Debug)]
pub enum ExchangeRequest {
    Connect((Hostname, IpAddr)),
    // Send(String),
    TrustedSignatures(HashSet<String>),
}

impl Operation for ExchangeRequest {
    type Output = ExchangeEvent;
}
