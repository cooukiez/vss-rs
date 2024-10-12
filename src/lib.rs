use std::error::Error;
use rand::distributions::{Bernoulli, Distribution};
use rand::thread_rng;
use crate::bvox::{read_bvox, write_bvox, BvoxHeader, DEFAULT_CHUNK_SIZE};
use crate::vox::{morton_decode_3d_grid, morton_encode_3d_grid};

mod bsvo;
mod svo;
mod vox;
mod bvox;
mod rle;

const CHUNK_RES: u32 = DEFAULT_CHUNK_SIZE;
const CHUNK_SIZE: u32 = DEFAULT_CHUNK_SIZE;

pub fn gen_rand_vox_grid(size: usize, probability_of_one: f64) -> Vec<u8> {
    let mut rng = thread_rng();
    let dist = Bernoulli::new(probability_of_one).unwrap();
    (0..size).map(|_| dist.sample(&mut rng) as u8).collect()
}

pub fn test_bvox_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0u8; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::default();
    write_bvox("test.bvox", &chunk_data, header)?;

    let (read_header, read_chunk_data) = read_bvox("test.bvox")?;

    let mut decoded_morton = vec![0u8; CHUNK_SIZE as usize];
    morton_decode_3d_grid(&read_chunk_data[0], CHUNK_RES, CHUNK_SIZE, &mut decoded_morton);

    for i in 0..CHUNK_SIZE {
        assert_eq!(chunk[i], decoded_morton[i]);
    }

    Ok(())
}

pub fn test_bsvo_read_write() -> Result<(), Box<dyn Error>> {
    let chunk = gen_rand_vox_grid(CHUNK_SIZE as usize, 0.1);

    let mut morton_chunk = vec![0u8; CHUNK_SIZE as usize];
    morton_encode_3d_grid(&chunk, CHUNK_RES, CHUNK_SIZE, &mut morton_chunk);

    let chunk_data = vec![morton_chunk.clone()];

    let header = BvoxHeader::default();
    write_bvox("test.bvox", &chunk_data, header)?;

    let mut header = BvoxHeader::new(CHUNK_RES, CHUNK_SIZE, true, false);
    write_bvox("svo_test_grid.bvox", &chunk_data, &header)?;

    let svo = Svo::new(&morton_chunk, CHUNK_RES);

    let mut bsvo_header = BsvoHeader::new(svo.max_depth, svo.root_res, true);
    write_bsvo("test.bsvo", &svo, &bsvo_header)?;

    let mut read_svo = Svo::default();
    let mut read_bsvo_header = BsvoHeader::default();
    read_bsvo("test.bsvo", &mut read_svo, &mut read_bsvo_header)?;

    for i in 0..read_svo.nodes.len() {
        assert_eq!(svo.nodes[i].data, read_svo.nodes[i].data);
        assert_eq!(svo.nodes[i].child_mask, read_svo.nodes[i].child_mask);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvox_rw() {
        assert_eq!(test_bvox_read_write(), Ok(()));
    }

    #[test]
    fn test_bsvo_rw() {
        assert_eq!(test_bsvo_read_write(), Ok(()));
    }

    #[test]
    fn sample_bvox_bsvo() {
        assert_eq!(sample_bvox_and_bsvo(), Ok(()));
    }

    #[test]
    fn simple_test() {
        assert_eq!(simple_test_data(), Ok(()));
    }
}
