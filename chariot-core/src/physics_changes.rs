use std::time::Instant;

pub enum PhysicsChangeType {
    NoTurningRight,
    IAmSpeed,
    ShoppingCart,
    InSpainButTheAIsSilent,
}

pub struct PhysicsChange {
    pub change_type: PhysicsChangeType,
    pub expiration_time: Instant,
}
