use std::collections::HashMap;

use svudko_common::resolver::Operation;

use crate::event::ExchangeEvent;

#[derive(Clone, Debug)]
pub enum ExchangeRequest {
    // Connect(String),
    // Send(String),
    TrustedHosts(HashMap<String, String>),
}

impl Operation for ExchangeRequest {
    type Output = ExchangeEvent;
}
