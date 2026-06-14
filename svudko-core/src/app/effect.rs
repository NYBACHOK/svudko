use crux_core::{macros::effect, render::RenderOperation};

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
}
