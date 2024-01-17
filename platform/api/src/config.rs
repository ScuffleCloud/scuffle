use std::net::SocketAddr;
use std::path::PathBuf;

use common::config::{S3BucketConfig, TlsConfig};

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct ApiConfig {
	/// Bind address for the API
	pub bind_address: SocketAddr,

	/// If we should use TLS for the API server
	pub tls: Option<TlsConfig>,

	/// Max profile picture upload size
	pub max_profile_picture_size: usize,
}

impl Default for ApiConfig {
	fn default() -> Self {
		Self {
			bind_address: "[::]:4000".parse().expect("failed to parse bind address"),
			tls: None,
			max_profile_picture_size: 5 * 1024 * 1024, // 5 MB
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct TurnstileConfig {
	/// The Cloudflare Turnstile site key to use
	pub secret_key: String,

	/// The Cloudflare Turnstile url to use
	pub url: String,
}

impl Default for TurnstileConfig {
	fn default() -> Self {
		Self {
			secret_key: "1x0000000000000000000000000000000AA".to_string(),
			url: "https://challenges.cloudflare.com/turnstile/v0/siteverify".to_string(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct JwtConfig {
	/// JWT secret
	pub secret: String,

	/// JWT issuer
	pub issuer: String,
}

impl Default for JwtConfig {
	fn default() -> Self {
		Self {
			issuer: "scuffle".to_string(),
			secret: "scuffle".to_string(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct ImageUploaderConfig {
	/// The S3 Bucket which contains the source images
	pub bucket: S3BucketConfig,

	/// Profile picture callback subject (can't contain `.`)
	pub profile_picture_callback_subject: String,

	/// Profile picture task priority, higher number means higher priority
	pub profile_picture_task_priority: i64,

	/// Public endpoint for downloads
	pub public_endpoint: String,
}

impl Default for ImageUploaderConfig {
	fn default() -> Self {
		Self {
			bucket: S3BucketConfig::default(),
			profile_picture_callback_subject: "scuffle-platform-image_processor-profile_picture".to_string(),
			profile_picture_task_priority: 2,
			public_endpoint: "https://images.scuffle.tv/scuffle-image-processor-public".to_string(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct VideoApiConfig {
	/// The address of the video api
	pub address: String,

	/// The TLS config for the video api
	pub tls: Option<TlsConfig>,

	/// Edge endpoint to pass to the player
	pub edge_endpoint: String,

	/// The organization id for the video api
	pub organization_id: ulid::Ulid,

	/// The access key for the video api
	pub access_key: ulid::Ulid,

	/// The secret key for the video api
	pub secret_key: ulid::Ulid,

	/// The playback key pair for the video api
	pub playback_keypair: Option<VideoApiPlaybackKeypairConfig>,
}

impl Default for VideoApiConfig {
	fn default() -> Self {
		Self {
			address: "localhost:9080".to_string(),
			tls: None,
			edge_endpoint: "https://edge.scuffle.dev".to_string(),
			organization_id: ulid::Ulid::nil(),
			access_key: ulid::Ulid::nil(),
			secret_key: ulid::Ulid::nil(),
			playback_keypair: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
pub struct VideoApiPlaybackKeypairConfig {
	/// The playback key pair id for the video api
	pub id: ulid::Ulid,

	/// Path to the playback private key for the video api
	pub private_key: PathBuf,
}
