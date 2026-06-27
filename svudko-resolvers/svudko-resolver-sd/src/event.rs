use crate::models::LocalService;

#[derive(Clone, Debug)]
pub enum ServiceDiscoveryEvent {
    None,
    AppearedService(LocalService),
    LostService(String),
}
