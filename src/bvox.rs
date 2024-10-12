use std::{
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    io,
    fs::{File, OpenOptions},
    path::Path
};
use serde_derive::{Deserialize, Serialize};
use crate::rle::{run_length_decode, run_length_encode};

pub const BVOX_VERSION: u8 = 2;
pub const CHUNK_SEPARATOR: u8 = u8::MAX;
pub const DEFAULT_CHUNK_RES: u32 = 256;
pub const DEFAULT_CHUNK_SIZE: u32 = DEFAULT_CHUNK_RES * DEFAULT_CHUNK_RES * DEFAULT_CHUNK_RES;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BvoxHeader {
    version: u8,
    chunk_res: u32,
    chunk_size: u32,
    run_length_encoded: bool,
    morton_encoded: bool,
}

impl BvoxHeader {
    pub fn new(chunk_res: u32, chunk_size: u32, run_length_encoded: bool, morton_encoded: bool) -> Self {
        Self {
            version: BVOX_VERSION,
            chunk_res,
            chunk_size,
            run_length_encoded,
            morton_encoded,
        }
    }
}

impl Default for BvoxHeader {
    fn default() -> Self {
        Self::new(DEFAULT_CHUNK_RES, DEFAULT_CHUNK_SIZE, false, false)
    }
}

//
// writing
//
pub fn write_empty_bvox(filename: &str, header: BvoxHeader) -> io::Result<()> {
    let mut header = header;
    header.version = BVOX_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(&serde_cbor::to_vec(&header).unwrap())?;
    writer.flush()?;

    Ok(())
}

pub fn write_bvox(
    filename: &str,
    chunk_data: &[Vec<u8>],
    header: BvoxHeader,
) -> io::Result<()> {
    let mut header = header;
    header.version = BVOX_VERSION;

    let path = Path::new(filename);
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(&serde_cbor::to_vec(&header).unwrap())?;

    for chunk in chunk_data {
        if chunk.len() != header.chunk_size as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "chunk is not the given size."));
        }

        if header.run_length_encoded {
            let encoded = run_length_encode(chunk);
            writer.write_all(&encoded)?;
        } else {
            writer.write_all(chunk)?;
        }

        writer.write_all(&[CHUNK_SEPARATOR])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn get_bvox_header(filename: &str) -> io::Result<BvoxHeader> {
    let path = Path::new(filename);
    let mut reader = BufReader::new(File::open(path)?);

    let header: BvoxHeader = serde_cbor::from_reader(&mut reader).unwrap();

    if header.version > BVOX_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "newer bvox reader version required for file."));
    }

    if header.version < BVOX_VERSION {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file version is outdated, use older bvox reader."));
    }

    Ok(header)
}

pub fn append_to_bvox(filename: &str, chunk: &[u8]) -> io::Result<()> {
    let header = get_bvox_header(filename)?;

    // open file again to continue writing
    let path = Path::new(filename);
    let mut writer = BufWriter::new(OpenOptions::new().write(true).append(true).open(path)?);

    // check chunk size
    if chunk.len() != header.chunk_size as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "chunk is not the given size."));
    }

    if header.run_length_encoded {
        let encoded = run_length_encode(chunk);
        writer.write_all(&encoded)?;
    } else {
        writer.write_all(chunk)?;
    }

    writer.write_all(&[CHUNK_SEPARATOR])?;
    writer.flush()?;
    Ok(())
}

//
// reading
//
pub fn read_bvox(filename: &str) -> io::Result<(BvoxHeader, Vec<Vec<u8>>)> {
    let header = get_bvox_header(filename)?;

    // open file again to continue reading
    let path = Path::new(filename);
    let mut reader = BufReader::new(File::open(path)?);

    // skip the already read header part
    reader.seek(SeekFrom::Start(size_of::<BvoxHeader>() as u64))?;

    let mut chunk_data = Vec::new();
    let mut chunk = Vec::new();

    // read byte by byte
    let mut byte = [0u8; 1];
    while reader.read(&mut byte)? != 0 {
        // check if chunk ended
        if byte[0] == CHUNK_SEPARATOR {
            if chunk.len() != header.chunk_size as usize {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "the read chunk is not the expected size."));
            }

            if header.run_length_encoded {
                let decoded = run_length_decode(&chunk)?;
                chunk_data.push(decoded);
            } else {
                chunk_data.push(chunk.clone());
            }

            chunk.clear();
        } else {
            chunk.push(byte[0]);
        }
    }

    Ok((header, chunk_data))
}