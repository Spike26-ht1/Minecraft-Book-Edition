//! Seeded, deterministic terrain generation for Book Edition.
//!
//! The generator is intentionally independent of Android and the renderer.
//! A world seed plus chunk coordinates and generator version fully determine
//! the generated section.

use book_core::CHUNK_EDGE;

use crate::{BlockId, Chunk};

/// Version of the terrain algorithm. Persist this with worlds so future
/// generator changes do not silently alter newly generated chunks.
pub const TERRAIN_GENERATOR_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    Mountains,
}

/// Immutable generation settings for one world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainGenerator {
    pub seed: u64,
}

impl TerrainGenerator {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generates one 16×16×16 vertical section at horizontal chunk (x, z)
    /// and vertical section coordinate section_y.
    pub fn generate_section(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Chunk {
        let mut chunk = Chunk::new();

        for local_x in 0..CHUNK_EDGE {
            for local_z in 0..CHUNK_EDGE {
                let world_x = chunk_x * CHUNK_EDGE + local_x;
                let world_z = chunk_z * CHUNK_EDGE + local_z;
                let biome = self.biome_at(world_x, world_z);
                let surface = self.surface_height(world_x, world_z);

                for local_y in 0..CHUNK_EDGE {
                    let world_y = section_y * CHUNK_EDGE + local_y;
                    let block = if world_y == 0 {
                        BlockId::BEDROCK
                    } else if world_y > surface {
                        BlockId::AIR
                    } else {
                        self.block_at_depth(biome, surface - world_y)
                    };

                    if !block.is_air() {
                        let _ = chunk.set_block(local_x, local_y, local_z, block);
                    }
                }
            }
        }

        chunk
    }

    pub fn biome_at(&self, world_x: i32, world_z: i32) -> Biome {
        let temperature = value_noise(self.seed ^ 0xA076_1D64_78BD_642F, world_x, world_z, 192);
        let moisture = value_noise(self.seed ^ 0xE703_7ED1_A0B4_28DB, world_x, world_z, 160);
        let elevation = value_noise(self.seed ^ 0x8EBC_6AF0_9C88_C6E3, world_x, world_z, 112);

        if elevation > 0.48 {
            Biome::Mountains
        } else if temperature > 0.28 && moisture < -0.08 {
            Biome::Desert
        } else if moisture > 0.18 {
            Biome::Forest
        } else {
            Biome::Plains
        }
    }

    pub fn surface_height(&self, world_x: i32, world_z: i32) -> i32 {
        let continent = value_noise(self.seed, world_x, world_z, 128);
        let hills = value_noise(self.seed ^ 0xD1B5_4A32_D192_ED03, world_x, world_z,  forty_eight());
        let detail = value_noise(self.seed ^ 0x94D0_49BB_1331_11EB, world_x, world_z, 24);
        let biome = self.biome_at(world_x, world_z);

        let biome_offset = match biome {
            Biome::Plains => 0.0,
            Biome::Forest => 2.0,
            Biome::Desert => -1.0,
            Biome::Mountains => 13.0,
        };

        (27.0 + continent * 10.0 + hills * 6.0 + detail * 2.0 + biome_offset)
            .round()
            .clamp(8.0, 62.0) as i32
    }

    fn block_at_depth(&self, biome: Biome, depth: i32) -> BlockId {
        match biome {
            Biome::Desert => {
                if depth < 5 { BlockId::SAND } else { BlockId::STONE }
            }
            Biome::Mountains => {
                if depth == 0 { BlockId::STONE } else if depth < 4 { BlockId::DIRT } else { BlockId::STONE }
            }
            Biome::Plains | Biome::Forest => {
                if depth == 0 { BlockId::GRASS } else if depth < 4 { BlockId::DIRT } else { BlockId::STONE }
            }
        }
    }
}

fn forty_eight() -> i32 {
    48
}

fn value_noise(seed: u64, x: i32, z: i32, scale: i32) -> f64 {
    let cell_x = x.div_euclid(scale);
    let cell_z = z.div_euclid(scale);
    let frac_x = x.rem_euclid(scale) as f64 / scale as f64;
    let frac_z = z.rem_euclid(scale) as f64 / scale as f64;
    let smooth_x = frac_x * frac_x * (3.0 - 2.0 * frac_x);
    let smooth_z = frac_z * frac_z * (3.0 - 2.0 * frac_z);

    let a = lattice_value(seed, cell_x, cell_z);
    let b = lattice_value(seed, cell_x + 1, cell_z);
    let c = lattice_value(seed, cell_x, cell_z + 1);
    let d = lattice_value(seed, cell_x + 1, cell_z + 1);

    let top = lerp(a, b, smooth_x);
    let bottom = lerp(c, d, smooth_x);
    lerp(top, bottom, smooth_z)
}

fn lattice_value(seed: u64, x: i32, z: i32) -> f64 {
    let mut value = seed
        ^ (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);

    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;

    let unit = (value >> 11) as f64 / ((1_u64 << 53) as f64);
    unit * 2.0 - 1.0
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_coordinates_generate_identical_sections() {
        let generator_a = TerrainGenerator::new(0xB00C_ED17);
        let generator_b = TerrainGenerator::new(0xB00C_ED17);
        let a = generator_a.generate_section(-2, 1, 4);
        let b = generator_b.generate_section(-2, 1, 4);

        for y in 0..CHUNK_EDGE {
            for z in 0..CHUNK_EDGE {
                for x in 0..CHUNK_EDGE {
                    assert_eq!(a.get_block(x, y, z), b.get_block(x, y, z));
                }
            }
        }
    }

    #[test]
    fn different_seeds_change_at_least_one_sampled_height() {
        let a = TerrainGenerator::new(1);
        let b = TerrainGenerator::new(2);
        let differs = (-32..32).any(|x| {
            (-32..32).any(|z| a.surface_height(x, z) != b.surface_height(x, z))
        });
        assert!(differs);
    }

    #[test]
    fn generated_world_has_bedrock_and_surface_blocks() {
        let generator = TerrainGenerator::new(42);
        let bottom = generator.generate_section(0, 0, 0);
        assert_eq!(bottom.get_block(0, 0, 0), BlockId::BEDROCK);

        let surface_y = generator.surface_height(0, 0);
        let section_y = surface_y.div_euclid(CHUNK_EDGE);
        let local_y = surface_y.rem_euclid(CHUNK_EDGE);
        let section = generator.generate_section(0, section_y, 0);
        assert!(!section.get_block(0, local_y, 0).is_air());
    }

    #[test]
    fn heights_stay_inside_the_world_generation_bounds() {
        let generator = TerrainGenerator::new(u64::MAX);
        for x in -128..128 {
            for z in -128..128 {
                let height = generator.surface_height(x, z);
                assert!((8..=62).contains(&height));
            }
        }
    }
}
