use std::{collections::HashSet, net::IpAddr, path::PathBuf};

use svudko_common::{hostname::Hostname, resolver::Operation};

use crate::event::ExchangeEvent;

#[derive(Clone, Debug)]
pub enum ExchangeRequest {
    Connect((Hostname, IpAddr)),
    SendFiles((Hostname, Vec<PathBuf>)),
    PairedDevices(HashSet<String>),
}

impl Operation for ExchangeRequest {
    type Output = ExchangeEvent;
}
