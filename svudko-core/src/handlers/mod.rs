use std::sync::Weak;

use crux_core::{Request, capability::Operation, effects::ResolveSink};
use tokio::sync::mpsc::UnboundedSender;

use crate::{TOKIO_RUNTIME, resolvers::HandlerResolver};

pub mod connection;
pub mod dns_sd;
