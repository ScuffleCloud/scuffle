pub trait MtlsInterface: Send + Sync {
    fn mtls_root_cert_pem(&self) -> &[u8];
    fn mtls_cert_pem(&self) -> &[u8];
    fn mtls_private_key_pem(&self) -> &[u8];
}
