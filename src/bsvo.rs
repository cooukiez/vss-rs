use std::{fs::File, io, io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::Path, ptr, slice};
use crate::svo::{DEFAULT_SVO_MAX_DEPTH, SVO};

pub const BSVO_VERSION: u8 = 3;
pub const NODE_SIZE: usize = size_of::<u32>();

// Todo: implement run length encoding for bsvo? is it worth it?
#[derive(Copy, Clone, Debug)]
pub struct BsvoHeader {
    version: u8,
    pub depth: u8,
    pub root_span: f32,
    #[allow(dead_code)]
    pub run_length_encoded: bool,
}

const BSVO_HEADER_SIZE: usize = size_of::<BsvoHeader>();

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

#[warn(dead_code)]
pub fn write_empty_bsvo(filename: &str, header: BsvoHeader) -> io::Result<()> {
    let mut header = header;
    header.version = BSVO_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    let header_bytes = unsafe { slice::from_raw_parts(&header as *const _ as *const u8, BSVO_HEADER_SIZE) };
    writer.write_all(header_bytes)?;

    writer.flush()?;

    Ok(())
}

pub fn write_bsvo(filename: &str, svo: &SVO, header: BsvoHeader) -> io::Result<()> {
    let mut header = header;
    header.version = BSVO_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    let header_bytes = unsafe { slice::from_raw_parts(&header as *const _ as *const u8, BSVO_HEADER_SIZE) };
    writer.write_all(header_bytes)?;

    for &node in &svo.nodes {
        let bytes = node.to_le_bytes();
        writer.write_all(&bytes)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn get_bsvo_header(filename: &str) -> io::Result<BsvoHeader> {
    let path = Path::new(filename);
    let mut reader = BufReader::new(File::open(path)?);

    let mut buffer = [0u8; BSVO_HEADER_SIZE];
    reader.read_exact(&mut buffer)?;
    let header: BsvoHeader = unsafe { ptr::read(buffer.as_ptr().cast()) };

    if header.version > BSVO_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "newer bsvo reader version required for file."));
    }

    if header.version < BSVO_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file version is outdated, use older bsvo reader."));
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

    let nodes_slice = unsafe { slice::from_raw_parts(buffer.as_ptr().cast(), node_count) };
    let nodes = nodes_slice.to_vec();

    let svo = SVO {
        nodes,
        root_span: header.root_span,
        depth: header.depth,
    };

    Ok((header, svo))
}