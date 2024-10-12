use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path
};
use serde_derive::{Deserialize, Serialize};
use std::slice::from_raw_parts;
use crate::svo::SVO;

pub const BSVO_VERSION: u8 = 3;
pub const NODE_SIZE: usize = size_of::<u32>();

#[repr(C, align(4))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BsvoHeader {
    pub version: u8,
    pub depth: u8,
    pub root_span: f32,
    pub run_length_encoded: bool,
}

pub fn write_empty_bsvo(filename: &str, header: BsvoHeader) -> io::Result<()> {
    let mut header = header;
    header.version = BSVO_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(&serde_cbor::to_vec(&header).unwrap())?;
    writer.flush()?;

    Ok(())
}

pub fn write_bsvo(filename: &str, svo: &SVO, header: BsvoHeader) -> io::Result<()> {
    let mut header = header;
    header.version = BSVO_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(&serde_cbor::to_vec(&header).unwrap())?;

    for node in &svo.nodes {
        writer.write_all(&serde_cbor::to_vec(&node).unwrap())?;
    }

    writer.flush()?;
    Ok(())
}

pub fn get_bsvo_header(filename: &str) -> io::Result<BsvoHeader> {
    let path = Path::new(filename);
    let mut reader = BufReader::new(File::open(path)?);

    let header: BsvoHeader = serde_cbor::from_reader(&mut reader).unwrap();

    if header.version > BSVO_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "newer bsvo reader version required for file"));
    }

    if header.version < BSVO_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file version is outdated, use older bsvo reader"));
    }

    Ok(header)
}

pub fn read_bsvo(filename: &str) -> io::Result<(SVO, BsvoHeader)> {
    let header = get_bsvo_header(filename)?;

    // open file again to continue reading
    let path = Path::new(filename);
    let mut reader = BufReader::new(File::open(path)?);

    // skip the already read header part
    reader.seek(SeekFrom::Start(size_of::<BsvoHeader>() as u64))?;

    // read rest of file
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    // check if buffer size contains correct data
    assert_eq!(buffer.len() % NODE_SIZE, 0);

    // deserialize each node from binary
    let node_count = buffer.len() / NODE_SIZE;

    let nodes_slice = unsafe { from_raw_parts(buffer.as_ptr() as *const u32, node_count) };
    let mut nodes = nodes_slice.to_vec();

    let svo = SVO {
        nodes,
        root_span: header.root_span,
        depth: header.depth,
    };

    Ok((svo, header))
}