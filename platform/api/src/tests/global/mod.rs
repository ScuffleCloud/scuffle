use std::{sync::Arc, time::Duration};

use crate::{config::AppConfig, global::GlobalState};
use common::{
    config::RedisSentinelConfig,
    context::{Context, Handler},
    logging,
    prelude::FutureTimeout,
};
use fred::types::ServerConfig;
use tokio::select;

pub mod turnstile;

pub async fn mock_global_state(mut config: AppConfig) -> (Arc<GlobalState>, Handler) {
    let (ctx, handler) = Context::new();

    dotenvy::dotenv().ok();

    logging::init(&config.logging.level, config.logging.mode)
        .expect("failed to initialize logging");

    let db = Arc::new(
        sqlx::PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
            .await
            .expect("failed to connect to database"),
    );

    let rmq = common::rmq::ConnectionPool::connect(
        std::env::var("RMQ_URL").expect("RMQ_URL not set"),
        lapin::ConnectionProperties::default(),
        Duration::from_secs(30),
        1,
    )
    .timeout(Duration::from_secs(5))
    .await
    .expect("failed to connect to rabbitmq")
    .expect("failed to connect to rabbitmq");

    let redis_config = fred::types::RedisConfig::from_url(
        std::env::var("REDIS_URL")
            .expect("REDIS_URL not set")
            .as_str(),
    )
    .expect("failed to parse redis url");

    config.redis.addresses = redis_config
        .server
        .hosts()
        .into_iter()
        .map(|x| x.to_string())
        .collect();
    config.redis.database = redis_config.database.unwrap_or_default();
    config.redis.password = redis_config.password;
    config.redis.username = redis_config.username;
    config.redis.sentinel = match redis_config.server {
        ServerConfig::Sentinel { service_name, .. } => Some(RedisSentinelConfig { service_name }),
        _ => None,
    };

    let redis = crate::global::setup_redis(&config).await;
    let redis_subscription =
        crate::global::setup_redis_subscription(&config, Default::default()).await;

    let global = Arc::new(GlobalState::new(config, db, rmq, redis, ctx));
    let global2 = global.clone();
    tokio::spawn(async move {
        select! {
            _ = global2.subscription_manager.run(global2.ctx.clone(), redis_subscription) => {},
            _ = global2.rmq.handle_reconnects() => {},
            _ = global2.ctx.done() => {},
        }
    });

    (global, handler)
}
