use crux_core::capability::Operation;

pub mod connection;
pub mod dns_sd;

pub trait HandlerResolver<Op, Out>: Send + Sync + 'static {
    fn resolve(&mut self, op: &Op) -> impl Future<Output = Out> + Send + Sync;
}
