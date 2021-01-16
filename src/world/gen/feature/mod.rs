use crate::rand::jrand::JavaRandom;
use crate::world::ChunkMap;


pub trait Feature {

    /// Generate the feature in this chunk, the x/y/z coordinates is where to generate it.
    ///
    /// When called from the biome decorator, `y=0` and x/z are the coordinates of the population
    /// chunk, a.k.a. the chunk with an offset of 8/8 blocks.
    fn generate(&self, world: &mut ChunkMap, rand: &mut JavaRandom, x: i32, y: i32, z: i32);

}


mod distrib;
mod repeated;

pub use distrib::*;
pub use repeated::*;
