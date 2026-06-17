use std::sync::Weak;

use crux_core::{Request, capability::Operation, effects::ResolveSink};
use tokio::sync::mpsc::UnboundedSender;

use svudko_common::ASYNC_RUNTIME;

pub mod dns_sd;
pub mod exchange;
