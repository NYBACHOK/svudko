use crux_core::Request;
use tokio::sync::mpsc::UnboundedSender;

use crate::event::ConnectionRequest;

use super::*;

pub struct Handler {
    jobs_tx: UnboundedSender<Request<ConnectionRequest>>,
}

impl Handler {
    pub fn new<R, I>(sink: Weak<R>, mut operator: I) -> Self
    where
        R: ResolveSink<ConnectionRequest> + Send + Sync + 'static,
        I: HandlerResolver<ConnectionRequest, <ConnectionRequest as Operation>::Output>,
    {
        let (jobs_tx, mut jobs_rx) =
            tokio::sync::mpsc::unbounded_channel::<Request<ConnectionRequest>>();

        TOKIO_RUNTIME.spawn({
            async move {
                while let Some(mut request) = jobs_rx.recv().await {
                    let output = operator.resolve(&request.operation).await;

                    if let Some(sink) = sink.upgrade() {
                        sink.resolve_request(&mut request, output)
                            .expect("background resolve should succeed");
                    }
                }
            }
        });

        Self { jobs_tx }
    }

    pub fn process(&self, request: Request<ConnectionRequest>) {
        self.jobs_tx.send(request).expect("worker disconnected");
    }
}
