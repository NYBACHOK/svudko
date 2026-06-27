use super::*;

pub fn handle(
    event: ServiceDiscoveryEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    tracing::debug!(method = "handle_dns_events", event = ?event);

    match event {
        ServiceDiscoveryEvent::None => return Command::done(),
        ServiceDiscoveryEvent::AppearedService(service) => {
            if service.fullname.contains(model.session_id.base64_repr()) {
                return Command::done();
            }

            model
                .discovered_services
                .insert(service.hostname.clone(), service);
        }
        ServiceDiscoveryEvent::LostService(fullname) => {
            let key =
                model
                    .discovered_services
                    .iter()
                    .find_map(|(hostname, service)| match service.fullname == fullname {
                        true => Some(hostname.clone()),
                        false => None,
                    });

            if let Some(key) = key {
                let _ = model.discovered_services.remove(&key);
            }
        }
    }

    render()
}
