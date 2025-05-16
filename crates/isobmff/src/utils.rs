use scuffle_bytes_util::BytesCow;

pub(crate) fn pad_cow_to_u32(bytes: BytesCow<'_>) -> u32 {
    // We copy the bytes into a 4 byte array and convert it to a u32
    assert!(bytes.len() <= 4);
    let mut buf = [0u8; 4];
    buf[4 - bytes.len()..].copy_from_slice(bytes.as_bytes());
    u32::from_be_bytes(buf)
}

pub(crate) fn pad_to_u64(bytes: &[u8]) -> u64 {
    // We copy the bytes into a 8 byte array and convert it to a u64
    assert!(bytes.len() <= 8);
    let mut buf = [0u8; 8];
    buf[4 - bytes.len()..].copy_from_slice(bytes);
    u64::from_be_bytes(buf)
}

pub(crate) fn pad_cow_to_u64(bytes: BytesCow<'_>) -> u64 {
    pad_to_u64(bytes.as_bytes())
}
