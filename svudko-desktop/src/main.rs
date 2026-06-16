// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::Arc};

use slint::ToSharedString;
use smol::channel::Sender;
use svudko_core::{ApplicationCore, CruxShell, Effect};

slint::include_modules!();

struct SlintShell {
    tx: Sender<svudko_core::Effect>,
}

impl CruxShell for SlintShell {
    fn process_effects(&self, effect: svudko_core::Effect) {
        let _ = self.tx.send(effect);
    }
}

impl From<svudko_core::view_model::ViewModel> for ViewModel {
    fn from(svudko_core::view_model::ViewModel {}: svudko_core::view_model::ViewModel) -> Self {
        Self {}
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let (tx, rx) = smol::channel::unbounded();

    let core = Arc::new(ApplicationCore::new(Arc::new(SlintShell { tx })));

    let app = AppWindow::new()?;

    let _ = slint::spawn_local({
        let app = app.clone_strong();
        let core = Arc::clone(&core);

        async move {
            while let Ok(effect) = rx.recv().await {
                match effect {
                    Effect::Render(_) => {
                        app.set_model(core.inner().view().into());
                    }
                    Effect::Error(e) => {
                        app.set_model(core.inner().view().into());
                        app.set_error_msg(e.to_shared_string());
                    }
                }
            }
        }
    });

    app.run()?;

    Ok(())
}

pub fn setup_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[allow(unused_mut)]
    let mut registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    match cfg!(debug_assertions) {
                        true => tracing::Level::DEBUG,
                        false => tracing::Level::INFO,
                    }
                    .into(),
                )
                .from_env()
                .expect("default level is set")
                .add_directive("hyper_util=warn".parse().unwrap())
                .add_directive("winit=info".parse().unwrap())
                .add_directive("sctk=info".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap()),
        );

    registry.init();
}
