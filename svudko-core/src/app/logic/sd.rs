use super::*;

pub fn handle(
    event: ServiceDiscoveryEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    tracing::debug!(method = "handle_dns_events", event = ?event);

    match event {
        ServiceDiscoveryEvent::Enabled => model.dns_sd.enabled_discover = true,
        ServiceDiscoveryEvent::Disabled => model.dns_sd.enabled_discover = false,
        ServiceDiscoveryEvent::FoundServices(services) => {
            model.dns_sd.discovered_services = services
        }
        ServiceDiscoveryEvent::FoundIps(_ips) => todo!(),
    }

    render()
}
