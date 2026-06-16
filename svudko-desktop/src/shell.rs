use slint::Weak;
use smol::channel::Sender;
use svudko_core::CruxShell;

use crate::AppWindow;

pub struct SlintShell {
    pub tx: Sender<svudko_core::Effect>,
    pub app: Weak<AppWindow>,
}

impl CruxShell for SlintShell {
    fn process_effects(&self, effect: svudko_core::Effect) {
        let _ = self
            .app
            .upgrade_in_event_loop({
                let tx = self.tx.clone();

                move |_| {
                    let _ = slint::spawn_local({
                        async move {
                            let _ = tx.send(effect).await.inspect_err(
                                |e| tracing::error!(e = %e, "failed to send effect from shell"),
                            );
                        }
                    });
                }
            })
            .inspect_err(
                |e| tracing::error!(e = %e, "failed to upgrade app weak for handling of effects"),
            );
    }
}
