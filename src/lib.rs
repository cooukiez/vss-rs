mod bsvo;
mod svo;

pub const SVO_VERSION: u8 = 2;
pub const BSVO_VERSION: u8 = 3;

// general
pub const DEFAULT_MAT: u32 = 1;

// svo
pub const DEFAULT_SVO_MAX_DEPTH: u8 = 8;
pub const CHILD_OFFSET: u32 = 24;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
