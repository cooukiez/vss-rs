use std::io;

pub const RLE_MAX: u8 = u8::MAX - 1;

pub fn run_length_encode(data: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut current = data[0];
    let mut count = 1;

    for &byte in &data[1..] {
        if byte == current {
            if count == RLE_MAX {
                encoded.push(current);
                encoded.push(count);
                count = 0;
            }
            count += 1;
        } else {
            encoded.push(current);
            encoded.push(count);
            current = byte;
            count = 1;
        }
    }

    encoded.push(current);
    encoded.push(count);
    encoded
}

pub fn run_length_decode(data: &[u8]) -> io::Result<Vec<u8>> {
    if data.len() % 2 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid encoded vector size."));
    }

    let mut decoded = Vec::new();

    for chunk in data.chunks(2) {
        let value = chunk[0];
        let count = chunk[1];
        decoded.extend(vec![value; count as usize]);
    }

    Ok(decoded)
}