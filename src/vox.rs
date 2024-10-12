use glam::UVec3;

pub const DEFAULT_VOX_MAT: u8 = 1;

pub const fn pos_to_index(x: u32, y: u32, z: u32, res: u32) -> u32 {
    x + y * res + z * res * res
}

pub fn index_to_pos(index: u32, res: u32) -> UVec3 {
    UVec3::new(
        index % res,
        (index / res) % res,
        index / (res * res)
    )
}

pub fn spread_bits(byte: u8) -> u32 {
    let mut x = byte as u32;
    x = (x | (x << 16)) & 0x030000FF;
    x = (x | (x << 8)) & 0x0300F00F;
    x = (x | (x << 4)) & 0x030C30C3;
    x = (x | (x << 2)) & 0x09249249;
    x
}

pub fn morton_encode_3d(x: u8, y: u8, z: u8) -> u32 {
    spread_bits(x) | (spread_bits(y) << 1) | (spread_bits(z) << 2)
}

pub fn compare_bits(mut x: u32) -> u8 {
    x &= 0x09249249;
    x = (x | (x >> 2)) & 0x030C30C3;
    x = (x | (x >> 4)) & 0x0300F00F;
    x = (x | (x >> 8)) & 0x030000FF;
    x = (x | (x >> 16)) & 0x000003FF;
    x as u8
}

pub fn morton_decode_3d(morton_code: u32) -> (u8, u8, u8) {
    let x = compare_bits(morton_code);
    let y = compare_bits(morton_code >> 1);
    let z = compare_bits(morton_code >> 2);
    (x, y, z)
}

pub fn morton_encode_3d_grid(grid: &[u8], res: u32, size: u32, morton_grid: &mut [u8]) {
    for i in 0..size {
        let pos = index_to_pos(i, res);
        morton_grid[morton_encode_3d(pos.x as u8, pos.y as u8, pos.z as u8) as usize] = grid[i as usize];
    }
}

pub fn morton_decode_3d_grid(morton_grid: &[u8], res: u32, size: u32, grid: &mut [u8]) {
    for i in 0..size {
        let (x, y, z) = morton_decode_3d(i);
        let index = pos_to_index(x as u32, y as u32, z as u32, res);
        grid[index as usize] = morton_grid[i as usize];
    }
}