use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    // Shell shared events

    // Core only events
    #[serde(skip)]
    #[facet(skip)]
    Error(String),
}
