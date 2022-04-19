use std::time::Instant;

pub enum PhysicsChangeType {
    NoTurningRight,
    IAmSpeed,
    ShoppingCart,
    InSpainButTheAIsSilent,
}

pub struct PhysicsChange {
    pub change_type: PhysicsChangeType,
    pub which_player: i8,
    pub expiration_time: Instant,
}
