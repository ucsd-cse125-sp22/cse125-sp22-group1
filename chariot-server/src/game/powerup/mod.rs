pub mod action;
pub mod pickups;

#[allow(dead_code)]
// Powerups are not implemented yet, but the backbone/example structure is here for future reference
#[derive(Clone, Copy)]
pub enum PowerUp {
    // Beneficial
    Coffee,

    // Detrimental
    ShockEm,
    WetFloorSign,
}
