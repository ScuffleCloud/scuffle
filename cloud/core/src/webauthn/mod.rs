use std::sync::Arc;

use base64::Engine;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use pkcs8::AssociatedOid;
use sha2::Digest;
use tonic_types::{ErrorDetails, StatusExt};

use crate::CoreConfig;
use crate::models::MfaWebauthnCredential;
use crate::schema::mfa_webauthn_credentials;
use crate::std_ext::ResultExt;

mod challenge;

pub(crate) use challenge::new_challenge;

/// <https://w3c.github.io/webauthn/#dictionary-client-data>
#[derive(Debug, serde_derive::Deserialize)]
struct CollectedClientData {
    #[serde(rename = "type")]
    ty: CollectedClientDataType,
    challenge: String,
}

#[derive(Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub(crate) enum CollectedClientDataType {
    #[serde(rename = "webauthn.create")]
    Create,
    #[serde(rename = "webauthn.get")]
    Get,
    #[serde(other)]
    Unknown,
}

type GenericSpki<'a> =
    pkcs8::spki::SubjectPublicKeyInfo<pkcs8::spki::der::AnyRef<'a>, pkcs8::spki::der::asn1::BitStringRef<'a>>;

fn verify_signature<'a, 'b, K, S>(spki: GenericSpki<'a>, message: &[u8], signature: &'b [u8]) -> Result<(), VerifyError<'a>>
where
    K: TryFrom<GenericSpki<'a>, Error = pkcs8::spki::Error> + signature::Verifier<S>,
    S: TryFrom<&'b [u8], Error = signature::Error>,
{
    let verifying_key = K::try_from(spki)?;
    let signature = S::try_from(signature)?;
    verifying_key.verify(message, &signature)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum VerifyError<'a> {
    #[error("failed to parse client data: {0}")]
    ClientDataParse(#[from] serde_json::Error),
    #[error("unexpected type: expected {expected:?}, got {actual:?}")]
    UnexpectedType {
        expected: CollectedClientDataType,
        actual: CollectedClientDataType,
    },
    #[error("failed to decode challenge: {0}")]
    InvalidChallenge(#[from] base64::DecodeError),
    #[error("failed to verify challenge")]
    VerifyChallenge(#[from] challenge::ChallengeError),
    #[error("failed to decode public key: {0}")]
    InvalidPublicKey(#[from] pkcs8::spki::Error),
    #[error("failed to verify signature: {0}")]
    InvalidSignature(#[from] signature::Error),
    #[error("unsupported algorithm: {0:?}")]
    UnsupportedAlgorithm(pkcs8::spki::AlgorithmIdentifier<pkcs8::spki::der::AnyRef<'a>>),
}

pub(crate) fn verify_challenge<'a, G: CoreConfig>(
    global: &Arc<G>,
    expected_type: CollectedClientDataType,
    spki_data: &'a [u8],
    assertion_response: &pb::scufflecloud::core::v1::AuthenticatorAssertionResponse,
) -> Result<(), VerifyError<'a>> {
    let client_data: CollectedClientData = serde_json::from_slice(&assertion_response.client_data_json)?;

    if client_data.ty != expected_type {
        return Err(VerifyError::UnexpectedType {
            expected: expected_type,
            actual: client_data.ty,
        });
    }

    let challenge = base64::prelude::BASE64_URL_SAFE.decode(client_data.challenge)?;
    challenge::verify_challenge(global, &challenge)?;

    let spki = GenericSpki::try_from(spki_data)?;

    let mut message = vec![];
    message.extend_from_slice(&assertion_response.authenticator_data);
    message.extend(sha2::Sha256::digest(&assertion_response.client_data_json));

    match spki.algorithm.oid {
        ed25519_dalek::pkcs8::ALGORITHM_OID => verify_signature::<ed25519_dalek::VerifyingKey, ed25519_dalek::Signature>(
            spki,
            &message,
            &assertion_response.signature,
        )?,
        p256::NistP256::OID => verify_signature::<p256::ecdsa::VerifyingKey, p256::ecdsa::Signature>(
            spki,
            &message,
            &assertion_response.signature,
        )?,
        <sha2::Sha256 as rsa::pkcs1v15::RsaSignatureAssociatedOid>::OID => verify_signature::<
            rsa::pkcs1v15::VerifyingKey<sha2::Sha256>,
            rsa::pkcs1v15::Signature,
        >(
            spki, &message, &assertion_response.signature
        )?,
        p384::NistP384::OID => verify_signature::<p384::ecdsa::VerifyingKey, p384::ecdsa::Signature>(
            spki,
            &message,
            &assertion_response.signature,
        )?,
        // TODO: wait for stable release 0.14.0
        // p521::NistP521::OID => verify_signature::<p384::ecdsa::VerifyingKey, p384::ecdsa::Signature>(
        //     spki,
        //     &message,
        //     &assertion_response.signature,
        // )?,
        _ => return Err(VerifyError::UnsupportedAlgorithm(spki.algorithm)),
    }

    Ok(())
}

pub(crate) async fn process_challenge<G: CoreConfig>(
    global: &Arc<G>,
    tx: &mut diesel_async::AsyncPgConnection,
    credential_id: &[u8],
    assertion_response: &pb::scufflecloud::core::v1::AuthenticatorAssertionResponse,
) -> Result<MfaWebauthnCredential, tonic::Status> {
    let Some(webauthn_pk) = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
        .filter(mfa_webauthn_credentials::dsl::credential_id.eq(&credential_id))
        .select(MfaWebauthnCredential::as_select())
        .first::<MfaWebauthnCredential>(tx)
        .await
        .optional()
        .into_tonic_internal_err("failed to query webauthn public key")?
    else {
        return Err(tonic::Status::with_error_details(
            tonic::Code::NotFound,
            "webauthn public key not found",
            ErrorDetails::new(),
        ));
    };

    // Verify the challenge
    verify_challenge(
        global,
        CollectedClientDataType::Get,
        &webauthn_pk.spki_data,
        assertion_response,
    )
    .into_tonic_err(
        tonic::Code::PermissionDenied,
        "failed to verify webauthn challenge",
        ErrorDetails::new(),
    )?;

    Ok(webauthn_pk)
}
