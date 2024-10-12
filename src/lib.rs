use crate::bsvo::{read_bsvo, write_bsvo, BsvoHeader};
use crate::bvox::{read_bvox, write_bvox, BvoxHeader, DEFAULT_CHUNK_RES, DEFAULT_CHUNK_SIZE};
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

pub fn test_bvox_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, true);
    write_bvox("test_bvox_rw.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("test_bvox_rw.bvox")?;

    let mut decoded_morton = vec![0; CHUNK_SIZE as usize];
    morton_decode_3d_grid(&read_chunk_data[0], CHUNK_RES, CHUNK_SIZE, &mut decoded_morton);

    for i in 0..CHUNK_SIZE {
        assert_eq!(chunk[i as usize], decoded_morton[i as usize]);
    }

    Ok(())
}

pub fn test_normal_and_rle_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);
    let chunk_data = vec![chunk];

    let header_normal = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, false, false);
    write_bvox("test_bvox_normal_rw.bvox", &chunk_data, header_normal)?;

    let header_rle = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, false);
    write_bvox("test_bvox_rle_rw.bvox", &chunk_data, header_rle)?;

    let (_, read_normal) = read_bvox("test_bvox_normal_rw.bvox")?;
    let (_, read_rle) = read_bvox("test_bvox_rle_rw.bvox")?;

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
    write_bvox("test_bsvo_rw.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("test_bsvo_rw.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], CHUNK_RES, SVO_MAX_DEPTH);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("test_bsvo_rw.bsvo", &svo, bsvo_header)?;

    let (_, read_svo) = read_bsvo("test_bsvo_rw.bsvo")?;

    for i in 0..read_svo.nodes.len() {
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
    write_bvox("cube.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("cube.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], CHUNK_RES, SVO_MAX_DEPTH);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("cube.bsvo", &svo, bsvo_header)?;

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
    write_bvox("tiny_grid.bvox", &chunk_data, header)?;

    let (_, read_chunk_data) = read_bvox("tiny_grid.bvox")?;

    let svo = SVO::from_grid(&read_chunk_data[0], chunk_res, depth);

    let bsvo_header = BsvoHeader::new(svo.depth, svo.root_span, true);
    write_bsvo("tiny_svo.bsvo", &svo, bsvo_header)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvox_rw() {
        test_bvox_read_write().unwrap();
    }

    #[test]
    fn test_bvox_normal_rle_rw() {
        test_normal_and_rle_read_write().unwrap();
    }

    #[test]
    fn test_bsvo_rw() {
        test_bsvo_read_write().unwrap();
    }

    #[test]
    fn sample_bvox_bsvo() {
        cube_grid_and_svo().unwrap();
    }

    #[test]
    fn simple_test() {
        tiny_grid_and_svo().unwrap();
    }
}
