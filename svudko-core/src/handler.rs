use std::sync::Weak;

use crux_core::{
    App, Request,
    capability::Operation,
    effects::{EffectRouter, ResolveSink},
};
use svudko_common::{ASYNC_RUNTIME, resolver::HandlerResolver};
use tokio::sync::mpsc::UnboundedSender;

use crate::{Application, EffectRoutes};

pub struct Handler<T: Operation> {
    jobs_tx: UnboundedSender<Request<T>>,
}

impl<T: Operation> Handler<T> {
    pub fn new<I>(
        sink: Weak<EffectRouter<Application, EffectRoutes>>,
        operator: Result<I, I::Err>,
    ) -> Self
    where
        I: HandlerResolver<Op = T, Err: Into<<Application as App>::Event> + Send>,
    {
        let (jobs_tx, mut jobs_rx) = tokio::sync::mpsc::unbounded_channel::<Request<T>>();

        ASYNC_RUNTIME.spawn({
            async move {
                let mut operator = if let Ok(operator) = operator {
                    operator
                } else {
                    let err = operator.err().expect("matched to err");

                    tracing::error!(err = ?err, "error in handler during initialization");

                    if let Some(sink) = sink.upgrade() {
                        sink.update(err.into());
                    }

                    return;
                };

                while let Some(mut request) = jobs_rx.recv().await {
                    let output = operator.resolve(&request.operation).await;

                    if let Some(sink) = sink.upgrade() {
                        match output {
                            Ok(output) => sink
                                .resolve_request(&mut request, output)
                                .expect("store resolve should succeed"),
                            Err(err) => sink.update(err.into()),
                        }
                    }
                }
            }
        });

        Self { jobs_tx }
    }

    pub fn process(&self, request: Request<T>) {
        // I don't want to app crash, so silently drop new events in broken resolver
        //
        // Still, look into some form of `Core` event for this in late future
        let _ = self.jobs_tx.send(request);
    }
}
