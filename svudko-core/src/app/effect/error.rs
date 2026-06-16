use crux_core::capability::Operation;

#[derive(Debug)]
pub struct CoreErrorEffect(pub String);

impl Operation for CoreErrorEffect {
    type Output = ();
}
