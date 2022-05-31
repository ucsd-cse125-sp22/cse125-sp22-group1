use std::num::Wrapping;

use crate::drawable::util::TransformUniform;

pub trait Rng<T> {
    fn next(&mut self) -> T;
}

pub struct Pcg32Rng {
    state: u64,
    inc: u64,
}

impl Default for Pcg32Rng {
    fn default() -> Self {
        Self {
            state: 0x1801_3CAD_3A48_3F72,
            inc: 0x51DB_FCDA_0D6B_21D4,
        }
    }
}

impl Rng<u32> for Pcg32Rng {
    fn next(&mut self) -> u32 {
        let oldstate = Wrapping(self.state);
        self.state = (oldstate * Wrapping(6_364_136_223_846_793_005u64) + Wrapping(self.inc | 1)).0;

        let xorshifted: u32 = (((oldstate >> 18usize) ^ oldstate) >> 27usize).0 as u32;
        let rot: u32 = (oldstate >> 59usize).0 as u32;

        (xorshifted >> rot) | (xorshifted << ((-(rot as i32)) & 31))
    }
}

impl Rng<f32> for Pcg32Rng {
    fn next(&mut self) -> f32 {
        let next_u32: u32 = self.next();
        let u = (next_u32 >> 9) | 0x3f80_0000u32;
        let f = bytemuck::cast::<u32, f32>(u);
        return f - 1.0;
    }
}

impl Rng<glam::Vec2> for Pcg32Rng {
    fn next(&mut self) -> glam::Vec2 {
        glam::vec2(self.next(), self.next())
    }
}

impl Rng<glam::Vec3> for Pcg32Rng {
    fn next(&mut self) -> glam::Vec3 {
        glam::vec3(self.next(), self.next(), self.next())
    }
}

// http://planning.cs.uiuc.edu/node198.html
impl Rng<glam::Quat> for Pcg32Rng {
    fn next(&mut self) -> glam::Quat {
        let uvw: glam::Vec3 = self.next();
        let [u, v, w] = uvw.to_array();
        glam::quat(
            f32::sqrt(1.0 - u) * f32::sin(2.0 * std::f32::consts::PI * v),
            f32::sqrt(1.0 - u) * f32::cos(2.0 * std::f32::consts::PI * v),
            f32::sqrt(u) * f32::sin(2.0 * std::f32::consts::PI * w),
            f32::sqrt(u) * f32::cos(2.0 * std::f32::consts::PI * w),
        )
    }
}
