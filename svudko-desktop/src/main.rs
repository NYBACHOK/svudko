// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, rc::Rc, sync::Arc};

use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use svudko_core::{ApplicationCore, Effect, event::Event};
use svudko_resolver_sd::request::ServiceDiscoveryRequest;

slint::include_modules!();

mod shell;

use self::shell::SlintShell;

impl From<svudko_core::view_model::ViewModel> for ViewModel {
    fn from(
        svudko_core::view_model::ViewModel {
            enabled_discover,
            discovered_services,
            unknown_signatures,
        }: svudko_core::view_model::ViewModel,
    ) -> Self {
        Self {
            enabled_discover,
            discovered_hosts: ModelRc::new(Rc::new(
                discovered_services
                    .into_iter()
                    .map(SharedString::from)
                    .collect::<VecModel<_>>(),
            )),
            pending_hosts: ModelRc::new(Rc::new(
                unknown_signatures
                    .into_iter()
                    .map(|(name, signature)| PendingHost {
                        name: name.to_shared_string(),
                        signature: signature.to_shared_string(),
                    })
                    .collect::<VecModel<_>>(),
            )),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let (tx, rx) = smol::channel::unbounded();

    let app = AppWindow::new()?;

    let core = Arc::new(ApplicationCore::new(Arc::new(SlintShell {
        tx,
        app: app.as_weak(),
    })));

    app.set_model(core.inner().view().into());

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
    })?;

    app.on_start_scan({
        let core = Arc::clone(&core);

        move || {
            core.inner().update(Event::ServiceDiscovery(
                ServiceDiscoveryRequest::BrowseForServices,
            ));
        }
    });

    // app.on_connect_to_host({
    //     let core = Arc::clone(&core);

    //     move |hostname| {
    //         core.inner()
    //             .update(Event::Exchange(ExchangeRequest::Connect(
    //                 hostname.to_string(),
    //             )));
    //     }
    // });

    app.on_enable_discover({
        let core = Arc::clone(&core);

        move || {
            core.inner().update(Event::ServiceDiscovery(
                ServiceDiscoveryRequest::EnableService,
            ));
        }
    });

    app.on_disable_discover({
        let core = Arc::clone(&core);

        move || {
            core.inner().update(Event::ServiceDiscovery(
                ServiceDiscoveryRequest::DisableService,
            ));
        }
    });

    // app.on_send_debug_file({
    //     let core = Arc::clone(&core);

    //     move |hostname| {
    //         core.inner()
    //             .update(Event::Exchange(ExchangeRequest::SendFile(
    //                 hostname.to_string(),
    //             )));
    //     }
    // });

    core.inner().update(Event::Initialize);
    app.run()?;

    Ok(())
}

pub fn setup_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    if cfg!(debug_assertions) {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::INFO
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
