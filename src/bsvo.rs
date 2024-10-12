use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path
};
use serde_derive::{Deserialize, Serialize};
use std::slice::from_raw_parts;
use crate::svo::{DEFAULT_SVO_MAX_DEPTH, SVO};

pub const BSVO_VERSION: u8 = 3;
pub const NODE_SIZE: usize = size_of::<u32>();

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BsvoHeader {
    version: u8,
    pub depth: u8,
    pub root_span: f32,
    pub run_length_encoded: bool,
}

impl BsvoHeader {
    pub fn new(depth: u8, root_span: f32, run_length_encoded: bool) -> BsvoHeader {
        Self {
            version: BSVO_VERSION,
            depth,
            root_span,
            run_length_encoded,
        }
    }
}

impl Default for BsvoHeader {
    fn default() -> Self {
        Self::new(DEFAULT_SVO_MAX_DEPTH, 2u32.pow(DEFAULT_SVO_MAX_DEPTH as u32) as f32, false)
    }
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

pub fn read_bsvo(filename: &str) -> io::Result<(BsvoHeader, SVO)> {
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
    let nodes = nodes_slice.to_vec();

    let svo = SVO {
        nodes,
        root_span: header.root_span,
        depth: header.depth,
    };

    Ok((header, svo))
}