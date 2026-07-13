use svudko_common::hostname::Hostname;

#[derive(Debug, Clone)]
pub struct ClientId {
    pub hostname: Hostname,
    pub id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ClientIdRaw {
    pub hostname: String,
    pub id: String,
}

impl From<ClientIdRaw> for ClientId {
    fn from(ClientIdRaw { hostname, id }: ClientIdRaw) -> Self {
        Self {
            hostname: Hostname::new(hostname),
            id,
        }
    }
}

impl From<ClientId> for ClientIdRaw {
    fn from(ClientId { hostname, id }: ClientId) -> Self {
        Self {
            hostname: hostname.into_inner(),
            id,
        }
    }
}
