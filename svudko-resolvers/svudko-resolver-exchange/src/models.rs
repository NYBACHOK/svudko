use svudko_common::hostname::Hostname;

#[derive(Debug, Clone)]
pub struct UnknownSignature {
    pub hostname: Hostname,
    pub signature: String,
}
