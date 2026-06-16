use crux_core::capability::Operation;

use crate::event::ConnectionEvent;

#[derive(Clone, Debug)]
pub enum ConnectionRequest {}

impl Operation for ConnectionRequest {
    type Output = Result<ConnectionEvent, String>;
}
