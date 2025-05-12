use mvutils::utils::Map;
use noise::{NoiseFn, Perlin};

pub struct GeneratorNoise {
    perlin: Perlin,
    scale: f64
}

impl GeneratorNoise {
    pub fn new(scale: f64, seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            scale,
        }
    }

    pub fn get_for_tile(&self, x: i32, z: i32) -> f64 {
        let val = self.perlin.get([x as f64 * self.scale + 0.5, z as f64 * self.scale + 0.5]);
        val.map(&(-1.0..1.0), &(0.0..1.0))
    }
}