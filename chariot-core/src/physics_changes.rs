use std::time::Instant;

#[derive(Clone)]
pub enum PhysicsChangeType {
    NoTurningRight,
    IAmSpeed,
    ShoppingCart,
    InSpainButTheAIsSilent,
}

#[derive(Clone)]
pub struct PhysicsChange {
    pub change_type: PhysicsChangeType,
    pub expiration_time: Instant,
}
