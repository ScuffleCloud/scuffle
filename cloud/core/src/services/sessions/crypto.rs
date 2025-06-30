use rand::TryRngCore;
use rsa::pkcs8::DecodePublicKey;
use tonic::Code;

use crate::std_ext::ResultExt;

pub(crate) fn generate_random_bytes() -> Result<[u8; 32], rand::rand_core::OsError> {
    let mut token = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut token)?;
    Ok(token)
}

pub(crate) fn encrypt_token(
    algorithm: pb::scufflecloud::core::v1::DeviceAlgorithm,
    token: &[u8],
    pk_der_data: &[u8],
) -> Result<Vec<u8>, tonic::Status> {
    match algorithm {
        pb::scufflecloud::core::v1::DeviceAlgorithm::RsaOaepSha256 => {
            let pk = rsa::RsaPublicKey::from_public_key_der(pk_der_data)
                .into_tonic(Code::InvalidArgument, "failed to parse public key")?;
            let padding = rsa::Oaep::new::<sha2::Sha256>();
            let enc_data = pk
                .encrypt(&mut rsa::rand_core::OsRng, padding, token)
                .into_tonic_internal("failed to encrypt token")?;
            Ok(enc_data)
        }
    }
}
