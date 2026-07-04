use std::path::PathBuf;

use svudko_common::hostname::Hostname;

#[derive(Clone, Debug)]
pub enum ExchangeRequestEvent {
    SendFiles((Hostname, Vec<PathBuf>)),
}
