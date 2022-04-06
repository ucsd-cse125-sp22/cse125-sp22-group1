pub const GRAVITY_COEFFICIENT: f64 = 1.0;

// Rolling resistance dominates at low-speed regimes and is proportional to
// velocity; drag dominates at higher speeds and is proportional to the square
// of velocity, so the rolling resistance coefficient must be much larger (~30x)
// than the drag coefficient
pub const DRAG_COEFFICIENT: f64 = 0.01;
pub const ROLLING_RESISTANCE_COEFFICIENT: f64 = 0.3;

pub const ROTATION_REDUCTION_COEFFICIENT: f64 = 0.95;

pub const CAR_ACCELERATOR: f64 = 1.0;
pub const CAR_BRAKE: f64 = 0.1;
pub const CAR_SPIN: f64 = 0.1;