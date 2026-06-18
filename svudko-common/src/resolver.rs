pub use crux_core::capability::Operation;

pub trait HandlerResolver: Send + Sync + 'static {
    /// Options to pass during resolver creation
    type Opt;
    /// Crux operation
    type Op: Operation;
    /// Error returned during processing or creation
    type Err: std::error::Error;

    fn new(opt: Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized;

    fn resolve(
        &mut self,
        op: &Self::Op,
    ) -> impl Future<Output = Result<<Self::Op as Operation>::Output, Self::Err>> + Send + Sync;
}
