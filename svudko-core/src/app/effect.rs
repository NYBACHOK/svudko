use crux_core::{macros::effect, render::RenderOperation};

use crate::resolvers::dns_sd::LocalDnsSdRequest;

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    DnsSd(LocalDnsSdRequest),
}
