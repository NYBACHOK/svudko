use crate::event::ConnectionRequest;

use super::*;

#[derive(Clone)]
pub struct Resolver {}

impl Resolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl HandlerResolver<ConnectionRequest, <ConnectionRequest as Operation>::Output> for Resolver {
    async fn resolve(
        &mut self,
        op: &ConnectionRequest,
    ) -> <ConnectionRequest as Operation>::Output {
        todo!()
    }
}
