pub trait HandlerResolver<Op, Out>: Send + Sync + 'static {
    type Opt;
    type Err;

    fn new(opt: Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized;

    fn resolve(&mut self, op: &Op) -> impl Future<Output = Out> + Send + Sync;
}
