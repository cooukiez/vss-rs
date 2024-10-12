use crate::bsvo::{read_bsvo, write_bsvo, write_empty_bsvo, BsvoHeader};
use crate::bvox::{append_to_bvox, read_bvox, write_bvox, write_empty_bvox, BvoxHeader, DEFAULT_CHUNK_RES, DEFAULT_CHUNK_SIZE};
use crate::svo::{DEFAULT_SVO_MAX_DEPTH, SVO};
use crate::vox::{morton_decode_3d_grid, morton_encode_3d_grid, pos_to_index, DEFAULT_VOX_MAT};
use rand::distributions::{Bernoulli, Distribution};
use rand::thread_rng;
use std::error::Error;

mod bsvo;
mod svo;
mod vox;
mod bvox;
mod rle;

const CHUNK_RES: u32 = DEFAULT_CHUNK_RES;
const CHUNK_SIZE: u32 = DEFAULT_CHUNK_SIZE;
const SVO_MAX_DEPTH: u8 = DEFAULT_SVO_MAX_DEPTH;

pub fn gen_rand_vox_grid(size: usize, probability_of_one: f64) -> Vec<u8> {
    let mut rng = thread_rng();
    let dist = Bernoulli::new(probability_of_one).unwrap();
    (0..size).map(|_| dist.sample(&mut rng) as u8).collect()
}

pub fn test_empty_bsvo_and_bvox() -> Result<(), Box<dyn Error>> {
    let bvox_header = BvoxHeader::default();
    write_empty_bvox("output/empty.bvox", bvox_header)?;

    let bsvo_header = BsvoHeader::default();
    write_empty_bsvo("output/empty.bsvo", bsvo_header)?;

    Ok(())
}

pub fn test_bvox_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, true);
    write_bvox("output/test_bvox_rw.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("output/test_bvox_rw.bvox")?;

    let mut decoded_morton = vec![0; CHUNK_SIZE as usize];
    morton_decode_3d_grid(&read_chunk_data[0], CHUNK_RES, CHUNK_SIZE, &mut decoded_morton);

    for i in 0..CHUNK_SIZE {
        assert_eq!(chunk[i as usize], decoded_morton[i as usize]);
    }

    Ok(())
}

pub fn test_bvox_append() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, true);
    write_empty_bvox("output/test_bvox_append.bvox", header)?;
    append_to_bvox("output/test_bvox_append.bvox", &morton_chunk)?;

    let (_, read_chunk_data) = read_bvox("output/test_bvox_append.bvox")?;

    let mut decoded_morton = vec![0; CHUNK_SIZE as usize];
    morton_decode_3d_grid(&read_chunk_data[0], CHUNK_RES, CHUNK_SIZE, &mut decoded_morton);

    for i in 0..CHUNK_SIZE {
        assert_eq!(chunk[i as usize], decoded_morton[i as usize]);
    }

    Ok (())
}

pub fn test_bvox_compression() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);
    let chunk_data = vec![chunk];

    let header_normal = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, false, false);
    write_bvox("output/test_bvox_compression_base.bvox", &chunk_data, header_normal)?;

    let header_rle = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, false);
    write_bvox("output/test_bvox_compression_rle.bvox", &chunk_data, header_rle)?;

    let (_, read_normal) = read_bvox("output/test_bvox_compression_base.bvox")?;
    let (_, read_rle) = read_bvox("output/test_bvox_compression_rle.bvox")?;

    for i in 0..CHUNK_SIZE {
        assert_eq!(read_normal[0][i as usize], read_rle[0][i as usize]);
    }

    Ok(())
}

pub fn test_bsvo_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0u8; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, true);
    write_bvox("output/test_bsvo_rw.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("output/test_bsvo_rw.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], CHUNK_RES, SVO_MAX_DEPTH);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("output/test_bsvo_rw.bsvo", &svo, bsvo_header)?;

    let (_, read_svo) = read_bsvo("output/test_bsvo_rw.bsvo")?;

    for i in 0..read_svo.nodes.len() {
        assert_eq!(svo.nodes[i], read_svo.nodes[i]);
    }

    let svo_node_count = svo.count_leaf_nodes();
    let chunk_node_count = chunk.iter().filter(|&&v| v > 0).count() as u32;
    assert_eq!(svo_node_count, chunk_node_count);

    Ok(())
}

pub fn test_gen_random_svo() -> Result<(), Box<dyn Error>> {
    let mut svo = SVO::new(SVO_MAX_DEPTH);
    svo.gen_random_svo(0);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, false);
    write_bsvo("output/random_svo.bsvo", &svo, bsvo_header)?;

    let (_, read_svo) = read_bsvo("output/random_svo.bsvo")?;

    for i in 0..svo.nodes.len() {
        assert_eq!(svo.nodes[i], read_svo.nodes[i]);
    }

    Ok(())
}

pub fn cube_grid_and_svo() -> Result<(), Box<dyn Error>> {
    let mut chunk = vec![0; CHUNK_SIZE as usize];
    let min = CHUNK_RES / 4;
    let max = 3 * CHUNK_RES / 4;

    for x in min..max {
        for y in min..max {
            for z in min..max {
                chunk[pos_to_index(x, y, z, CHUNK_RES) as usize] = DEFAULT_VOX_MAT;
            }
        }
    }

    let mut morton_chunk = vec![0u8; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, true);
    write_bvox("output/cube.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("output/cube.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], CHUNK_RES, SVO_MAX_DEPTH);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("output/cube.bsvo", &svo, bsvo_header)?;

    Ok (())
}

pub fn tiny_grid_and_svo() -> Result<(), Box<dyn Error>> {
    let chunk_res = 8;
    let chunk_size = chunk_res * chunk_res * chunk_res;
    let depth = 3;

    let mut chunk = vec![0; chunk_size as usize];
    let min = 2;
    let max = 4;

    for x in min..max {
        for y in min..max {
            for z in min..max {
                chunk[pos_to_index(x, y, z, chunk_res) as usize] = DEFAULT_VOX_MAT;
            }
        }
    }

    let mut morton_chunk = vec![0; chunk_size as usize];
    morton_encode_3d_grid(&chunk, chunk_res, chunk_size, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::new(chunk_res, chunk_size, true, true);
    write_bvox("output/tiny_grid.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("output/tiny_grid.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], chunk_res, depth);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("output/tiny_svo.bsvo", &svo, bsvo_header)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_empty() {
        test_empty_bsvo_and_bvox().unwrap();
    }

    #[test]
    fn bvox_append() {
        test_bvox_append().unwrap();
    }

    #[test]
    fn random_svo() {
        test_gen_random_svo().unwrap();
    }

    #[test]
    fn bvox_rw() {
        test_bvox_read_write().unwrap();
    }

    #[test]
    fn bvox_compression() {
        test_bvox_compression().unwrap();
    }

    #[test]
    fn bsvo_rw() {
        test_bsvo_read_write().unwrap();
    }

    #[test]
    fn cube() {
        cube_grid_and_svo().unwrap();
    }

    #[test]
    fn tiny_grid_and_svo_for_testing() {
        tiny_grid_and_svo().unwrap();
    }
}
