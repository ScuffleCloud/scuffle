use super::impl_enum;

impl_enum!(CryptoAlgorithm, crate::schema::sql_types::CryptoAlgorithm, {
    ED25519 => b"ED25519", // EdDSA using Ed25519 curve
    ESP256 => b"ESP256", // ECDSA using P-256 curve and SHA-256
    RS256 => b"RS256", // RSASSA-PKCS1-v1_5 using SHA-256
    ESP384 => b"ESP384", // ECDSA using P-384 curve and SHA-384
    ESP512 => b"ESP512" // ECDSA using P-521 curve and SHA-512
});
