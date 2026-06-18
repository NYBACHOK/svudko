#[derive(Debug, Clone)]
pub enum ExchangeRequest {
    Connect(String),
    SendFile(String),
}
