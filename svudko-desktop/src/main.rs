// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, rc::Rc, sync::Arc};

use slint::{ModelRc, ToSharedString, VecModel};
use svudko_core::{ApplicationCore, Effect, event::Event};

slint::include_modules!();

mod shell;

use self::shell::SlintShell;

impl From<svudko_core::view_model::ViewModel> for ViewModel {
    fn from(
        svudko_core::view_model::ViewModel {
            discovered_services,
            pairing_requests,
        }: svudko_core::view_model::ViewModel,
    ) -> Self {
        Self {
            discovered_hosts: ModelRc::new(Rc::new(
                discovered_services
                    .into_iter()
                    .map(LocalDevices::from)
                    .collect::<VecModel<_>>(),
            )),
        }
    }
}

impl From<svudko_core::view_model::LocalDevices> for LocalDevices {
    fn from(
        svudko_core::view_model::LocalDevices{ hostname, paired }: svudko_core::view_model::LocalDevices,
    ) -> Self {
        Self {
            hostname: Hostname {
                name: hostname.into_inner().to_shared_string(),
            },
            paired,
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

    app.global::<Logic<'_>>().set_model(core.view().into());

    let _ = slint::spawn_local({
        let app = app.clone_strong();
        let core = Arc::clone(&core);

        async move {
            while let Ok(effect) = rx.recv().await {
                match effect {
                    Effect::Render(_) => {
                        app.global::<Logic<'_>>().set_model(core.view().into());
                    }
                    Effect::Error(e) => {
                        app.global::<Logic<'_>>().set_model(core.view().into());
                        app.global::<Logic<'_>>()
                            .set_error_msg(e.to_shared_string());
                    }
                }
            }
        }
    })?;

    // app.on_select_files({
    //     let core = Arc::clone(&core);

    //     move |hostname| {
    //         let _ = slint::spawn_local({
    //             let core = Arc::clone(&core);

    //             async move {
    //                 let dialog = rfd::AsyncFileDialog::new()
    //                     .set_directory(dirs::home_dir().expect("always valid"))
    //                     .pick_files()
    //                     .await;

    //                 if let Some(files) = dialog {
    //                     core.inner()
    //                         .update(Event::Exchange(ExchangeRequestEvent::SendFiles((
    //                             hostname.to_string().into(),
    //                             files.into_iter().map(PathBuf::from).collect(),
    //                         ))));
    //                 }
    //             }
    //         });
    //     }
    // });

    core.update(Event::Initialize);
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
