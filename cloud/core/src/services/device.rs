use std::borrow::Cow;

use base64::Engine;
use core_db_types::models::{DeviceAlgorithm, NewDevice, Sha256};
use core_db_types::schema::devices;
use core_pb::v1::device_service_server::DeviceService;
use core_pb::v1::{Device, DeviceFinalizeRequest, DeviceFinalizeResponse, DeviceSetupRequest, DeviceSetupResponse, device};
use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{ChronoDateTimeExt, OptionExt, RequestExt, ResultExt};
use fred::prelude::*;
use pkcs8::{DecodePublicKey, LineEnding};
use rand::RngCore;
use rsa::pkcs1::EncodeRsaPublicKey;
use sha2::Digest;
use tonic::async_trait;
use tonic_types::{ErrorDetails, StatusExt};

const DEVICE_SETUP_EXPIRY: i64 = 600; // 10 minutes

fn proto_algorithm_to_db(algorithm: device::Algorithm) -> Result<DeviceAlgorithm, tonic::Status> {
    match algorithm {
        device::Algorithm::DeviceAlgorithmRsaOaepSha256 => Ok(DeviceAlgorithm::RsaOaepSha256),
        device::Algorithm::DeviceAlgorithmUnknown => Err(tonic::Status::with_error_details(
            tonic::Code::InvalidArgument,
            "invalid algorithm",
            ErrorDetails::with_bad_request_violation("algorithm", "unknown algorithm"),
        )),
    }
}

fn db_algorithm_to_proto(algorithm: DeviceAlgorithm) -> device::Algorithm {
    match algorithm {
        DeviceAlgorithm::RsaOaepSha256 => device::Algorithm::DeviceAlgorithmRsaOaepSha256,
    }
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
struct SetupData<'a> {
    algorithm: DeviceAlgorithm,
    public_key: Cow<'a, str>,
    code: Cow<'a, str>,
}

#[async_trait]
impl<G: core_traits::Global> DeviceService for crate::services::CoreSvc<G> {
    async fn setup(
        &self,
        request: tonic::Request<DeviceSetupRequest>,
    ) -> tonic::Result<tonic::Response<DeviceSetupResponse>> {
        let global = request.global::<G>()?;
        let req = request.into_inner();

        // Validate and parse algorithm
        let algorithm = device::Algorithm::try_from(req.algorithm)
            .into_tonic_err_with_field_violation("algorithm", "invalid algorithm")?;
        let algorithm = proto_algorithm_to_db(algorithm)?;

        // Validate public key by parsing it
        let (fingerprint, public_key) = match algorithm {
            DeviceAlgorithm::RsaOaepSha256 => {
                let key = rsa::RsaPublicKey::from_public_key_pem(&req.public_key_data)
                    .into_tonic_err_with_field_violation("public_key_data", "invalid RSA public key")?;
                let der = key
                    .to_pkcs1_der()
                    .into_tonic_internal_err("failed to convert public key to DER")?;
                let public_key = key
                    .to_pkcs1_pem(LineEnding::LF)
                    .map(Cow::Owned)
                    .into_tonic_internal_err("failed to convert public key to PEM")?;
                (Sha256::new(der.as_bytes()), public_key)
            }
        };

        // Generate random code (16 bytes)
        let mut code_bytes = [0u8; 16];
        rand::rngs::OsRng
            .try_fill_bytes(&mut code_bytes)
            .into_tonic_internal_err("failed to generate random code")?;
        let code = hex::encode(code_bytes);

        let device_fingerprint = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(fingerprint.0);

        // Store setup data in Redis
        let redis_key = format!("device:setup:{}", device_fingerprint);
        let setup_data = SetupData {
            algorithm,
            public_key,
            code: Cow::Borrowed(&code),
        };

        let _: () = global
            .redis()
            .set(
                &redis_key,
                serde_json::to_string(&setup_data).into_tonic_internal_err("failed to serialize setup data")?,
                Some(Expiration::EX(DEVICE_SETUP_EXPIRY)),
                None,
                false,
            )
            .await
            .into_tonic_internal_err("failed to store device setup data")?;

        Ok(tonic::Response::new(DeviceSetupResponse {
            device_fingerprint,
            code,
        }))
    }

    async fn finalize(
        &self,
        request: tonic::Request<DeviceFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<DeviceFinalizeResponse>> {
        let global = request.global::<G>()?;
        let req = request.into_inner();

        // Validate fingerprint format
        let fingerprint_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&req.device_fingerprint)
            .into_tonic_err_with_field_violation("device_fingerprint", "invalid base64 encoding")?;

        Sha256::from_hash(
            fingerprint_bytes
                .as_slice()
                .try_into()
                .into_tonic_err_with_field_violation("device_fingerprint", "invalid fingerprint")?,
        );

        // Retrieve setup data from Redis
        let redis_key = format!("device:setup:{}", req.device_fingerprint);
        let setup_data: Option<String> = global
            .redis()
            .getdel(&redis_key)
            .await
            .into_tonic_internal_err("failed to retrieve device setup data")?;

        let setup_data = setup_data.into_tonic_not_found("device setup not found or expired")?;

        let setup_data: SetupData =
            serde_json::from_str(&setup_data).into_tonic_internal_err("failed to parse setup data")?;

        // Verify signature
        match setup_data.algorithm {
            DeviceAlgorithm::RsaOaepSha256 => {
                let public_key = rsa::RsaPublicKey::from_public_key_pem(&setup_data.public_key)
                    .into_tonic_internal_err("failed to parse public key")?;

                // RSA-PSS with SHA256
                let padding = rsa::pss::Pss::new::<sha2::Sha256>();

                let hashed = sha2::Sha256::digest(setup_data.code.as_bytes());

                public_key
                    .verify(padding, &hashed, &req.code_signature)
                    .into_tonic_err_with_field_violation("signed_code", "signature verification failed")?;
            }
        }

        // Insert device into database
        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let new_device = NewDevice::builder(setup_data.algorithm, Cow::Borrowed(setup_data.public_key.as_bytes())).build();

        let device = diesel::insert_into(devices::table)
            .values(&new_device)
            .on_conflict(devices::fingerprint)
            .do_update()
            .set(devices::last_active_at.eq(chrono::Utc::now()))
            .returning(core_db_types::models::Device::as_select())
            .get_result::<core_db_types::models::Device>(&mut db)
            .await
            .into_tonic_internal_err("failed to insert device")?;

        Ok(tonic::Response::new(DeviceFinalizeResponse {
            device: Some(Device {
                fingerprint: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(device.fingerprint.0),
                algorithm: db_algorithm_to_proto(device.algorithm).into(),
                created_at: Some(device.created_at.to_prost_timestamp_utc()),
            }),
        }))
    }
}
