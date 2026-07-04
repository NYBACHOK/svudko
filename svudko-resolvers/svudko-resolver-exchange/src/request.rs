use std::{collections::HashSet, net::IpAddr, path::PathBuf};

use svudko_common::{hostname::Hostname, resolver::Operation};

use crate::{event::ExchangeEvent, models::ClientId};

#[derive(Clone, Debug)]
pub enum ExchangeRequest {
    UpdateClientId(ClientId),
    SendFiles((Hostname, IpAddr, Vec<PathBuf>)),
    PairedDevices(HashSet<String>),
}

impl Operation for ExchangeRequest {
    type Output = ExchangeEvent;
}
