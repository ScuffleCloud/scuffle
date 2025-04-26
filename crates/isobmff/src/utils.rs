pub(crate) fn pad_to_u64(bytes: &[u8]) -> u64 {
    // We copy the bytes into a 8 byte array and convert it to a u64
    assert!(bytes.len() <= 8);
    let mut buf = [0u8; 8];
    buf[8 - bytes.len()..].copy_from_slice(bytes);
    u64::from_be_bytes(buf)
}
