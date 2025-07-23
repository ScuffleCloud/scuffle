use std::sync::Arc;

use hmac::{Hmac, Mac};
use prost::{self, Message};
use sha2::Sha256;

use crate::chrono_ext::ChronoDateTimeExt;
use crate::prost_ext::ProstTimestampExt;
use crate::{CoreConfig, common};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChallengeError {
    #[error("invalid secret length: {0}")]
    InvalidSecretLength(#[from] hmac::digest::InvalidLength),
    #[error("failed to generated nonce: {0}")]
    Nonce(#[from] rand::rand_core::OsError),
    #[error("failed to decode challenge: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("failed to verify challenge: {0}")]
    Verify(hmac::digest::MacError),
    #[error("semantically invalid challenge")]
    Invalid,
    #[error("challenge expired at {0}")]
    Expired(chrono::DateTime<chrono::Utc>),
}

pub(crate) fn new_challenge<G: CoreConfig>(global: &Arc<G>) -> Result<Vec<u8>, ChallengeError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(global.webauthn_challenge_secret())?;

    let created_at = chrono::Utc::now();
    let challenge = pb::scufflecloud::core::v1::WebauthnChallenge {
        nonce: common::generate_random_bytes()?.to_vec(),
        created_at: Some(created_at.to_prost_timestamp_utc()),
        expires_at: Some((created_at + global.mfa_timeout()).to_prost_timestamp_utc()),
    }
    .encode_to_vec();

    mac.update(&challenge);

    let signed_challenge = pb::scufflecloud::core::v1::SignedWebauthnChallenge {
        challenge,
        signature: mac.finalize().into_bytes().to_vec(),
    };

    Ok(signed_challenge.encode_to_vec())
}

pub(crate) fn verify_challenge<G: CoreConfig>(global: &Arc<G>, challenge: &[u8]) -> Result<(), ChallengeError> {
    let signed_challenge = pb::scufflecloud::core::v1::SignedWebauthnChallenge::decode(challenge)?;

    let mut mac = Hmac::<Sha256>::new_from_slice(global.webauthn_challenge_secret())?;
    mac.update(&signed_challenge.challenge);
    mac.verify_slice(&signed_challenge.signature)
        .map_err(ChallengeError::Verify)?;

    let challenge = pb::scufflecloud::core::v1::WebauthnChallenge::decode(signed_challenge.challenge.as_slice())?;

    if challenge.created_at.ok_or(ChallengeError::Invalid)?.to_chrono() > chrono::Utc::now() {
        return Err(ChallengeError::Invalid);
    }

    let expires_at = challenge.expires_at.ok_or(ChallengeError::Invalid)?.to_chrono();
    if expires_at < chrono::Utc::now() {
        return Err(ChallengeError::Expired(expires_at));
    }

    Ok(())
}
