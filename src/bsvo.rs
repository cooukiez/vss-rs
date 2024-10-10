use std::error::Error;
use std::fs::File;
use std::io::Write;
use crate::BSVO_VERSION;

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
struct BsvoHeader {
    version: u8,
    max_depth: u8,
    root_res: u32,
    run_length_encoded: bool,
}
fn write_empty_bsvo(filename: &str, mut header: BsvoHeader) -> Result<(), Box<dyn Error>> {
    header.version = BSVO_VERSION;

    #[cfg(debug_assertions)]
    println!(
        "writing empty bsvo file: {} | version: {} | max depth: {} | root res: {} | rle: {}",
        filename,
        header.version,
        header.max_depth,
        header.root_res,
        header.run_length_encoded
    );

    let mut file = File::create(filename)?;
    file.write_all(unsafe { std::slice::from_raw_parts((&header as *const BsvoHeader) as *const u8, size_of::<BsvoHeader>()) })?;
    file.flush()?;
    Ok(())
}


fn write_bsvo(filename: &str, svo: &Svo, mut header: BsvoHeader) -> Result<(), Box<dyn Error>> {
    header.version = BSVO_VERSION;

    #[cfg(debug_assertions)]
    println!(
        "writing bsvo file: {} | version: {} | max depth: {} | root res: {} | rle: {}",
        filename,
        header.version,
        header.max_depth,
        header.root_res,
        header.run_length_encoded
    );

    let mut file = File::create(filename)?;
    file.write_all(unsafe { std::slice::from_raw_parts((&header as *const BsvoHeader) as *const u8, size_of::<BsvoHeader>()) })?;
    for node in &svo.nodes {
        file.write_all(unsafe { std::slice::from_raw_parts((node as *const SvoNode) as *const u8, size_of::<SvoNode>()) })?;
    }
    file.flush()?;
    Ok(())
}

