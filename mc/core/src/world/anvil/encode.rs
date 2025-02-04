use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;

use crate::heightmap::HeightmapType;
use crate::world::chunk::Chunk;
use crate::block::BlockState;
use crate::util::{PackedArray, PackedIterator};

use nbt::encode::write_compound_tag;
use nbt::CompoundTag;


/// Decode the NBT data from a reader and delegate chunk decoding to `decode_chunk`.
pub fn encode_chunk_to_writer(writer: &mut impl Write, chunk: &Chunk) {
    let mut root = CompoundTag::new();
    encode_chunk(&mut root, chunk);
    write_compound_tag(writer, &root).unwrap();
}

pub fn encode_chunk(tag_root: &mut CompoundTag, chunk: &Chunk) {

    let (cx, cz) = chunk.get_position();

    tag_root.insert_i32("DataVersion", 2586);

    tag_root.insert_compound_tag("Level", {

        let mut tag_level = CompoundTag::new();

        tag_level.insert_i32("xPos", cx);
        tag_level.insert_i32("zPos", cz);
        // Core crate don't support status, support for status is not planned in near future
        // because chunks should be generated independently, this is opposed to Notchian gen.
        tag_level.insert_str("Status", "full");

        tag_level.insert_i32_vec("Biomes", {
            chunk.iter_biomes().map(|biome| biome.get_id()).collect()
        });

        tag_level.insert_compound_tag_vec("Sections", {
            chunk.iter_loaded_sub_chunks()
                .map(|(cy, sc)| {

                    let mut tag_section = CompoundTag::new();
                    tag_section.insert_i8("Y", cy);

                    let mut palette_indices = HashMap::new();
                    let mut tag_palette = Vec::new();
                    let mut packed_blocks = PackedArray::new(4096, 4, None);

                    for (idx, state) in sc.iter_blocks().enumerate() {
                        let sid = match palette_indices.entry(state.get_key()) {
                            Entry::Occupied(o) => *o.get(),
                            Entry::Vacant(v) => {
                                tag_palette.push(encode_block_state(state));
                                *v.insert(tag_palette.len() - 1)
                            }
                        };
                        packed_blocks.set_with_resize(idx, sid as u64);
                    }

                    tag_section.insert_compound_tag_vec("Palette", tag_palette);
                    tag_section.insert_i64_vec("BlockStates", packed_blocks.into_inner()
                        .into_iter()
                        .map(|val| val as i64)
                        .collect());

                    tag_section

                })
        });

        let mut tag_heightmaps = CompoundTag::new();

        for heightmap_type in chunk.get_env().heightmaps.iter_heightmap_types() {
            if let Some(arr) = encode_heightmap(chunk, heightmap_type) {
                tag_heightmaps.insert_i64_vec(heightmap_type.get_name(), arr);
            }
        }

        tag_level

    });

}

pub fn encode_block_state(state: &'static BlockState) -> CompoundTag {

    let block = state.get_block();
    let mut tag_block = CompoundTag::new();
    tag_block.insert_str("Name", block.get_name());

    if let Some(it) = state.iter_raw_states() {
        let mut tag_props = CompoundTag::new();
        for (name, value) in it {
            tag_props.insert_str(name, value);
        }
        tag_block.insert_compound_tag("Properties", tag_props);
    }

    tag_block

}

pub fn encode_heightmap(chunk: &Chunk, heightmap_type: &'static HeightmapType) -> Option<Vec<i64>> {
    let (byte_size, it) = chunk.iter_heightmap_raw_columns(heightmap_type)?;
    Some(it.pack_aligned(byte_size).map(|v| v as i64).collect())
}
