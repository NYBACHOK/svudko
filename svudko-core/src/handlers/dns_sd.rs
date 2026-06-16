use crate::event::LocalDnsSdRequest;

use super::*;

pub struct Handler {
    jobs_tx: UnboundedSender<Request<LocalDnsSdRequest>>,
}

impl Handler {
    pub fn new<R, I>(sink: Weak<R>, mut operator: I) -> Self
    where
        R: ResolveSink<LocalDnsSdRequest> + Send + Sync + 'static,
        I: HandlerResolver<LocalDnsSdRequest, <LocalDnsSdRequest as Operation>::Output>,
    {
        let (jobs_tx, mut jobs_rx) =
            tokio::sync::mpsc::unbounded_channel::<Request<LocalDnsSdRequest>>();

        TOKIO_RUNTIME.spawn({
            async move {
                while let Some(mut request) = jobs_rx.recv().await {
                    let output = operator.resolve(&request.operation).await;

                    if let Some(sink) = sink.upgrade() {
                        sink.resolve_request(&mut request, output)
                            .expect("store resolve should succeed");
                    }
                }
            }
        });

        Self { jobs_tx }
    }

    pub fn process(&self, request: Request<LocalDnsSdRequest>) {
        self.jobs_tx.send(request).expect("worker disconnected");
    }
}
