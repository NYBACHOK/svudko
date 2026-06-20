use svudko_common::hostname::Hostname;

#[derive(Clone, Debug)]
pub enum ExchangeRequestEvent {
    Connect(Hostname),
}
