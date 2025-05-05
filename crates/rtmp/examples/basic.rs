use std::io::Cursor;

use scuffle_rtmp::ServerSession;
use scuffle_rtmp::session::server::{ServerSessionError, SessionData, SessionHandler};
use tokio::net::TcpListener;
use tracing::Instrument;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct Handler;

impl SessionHandler for Handler {
    async fn on_data(&mut self, _stream_id: u32, data: SessionData) -> Result<(), ServerSessionError> {
        match data {
            SessionData::Audio { data, .. } => {
                let tag = scuffle_flv::audio::AudioData::demux(&mut Cursor::new(data)).unwrap();
                tracing::info!("audio: {:?}", tag);
            }
            SessionData::Video { data, .. } => {
                let tag = scuffle_flv::video::VideoData::demux(&mut Cursor::new(data)).unwrap();
                tracing::info!("video: {:?}", tag);
            }
            SessionData::Amf0 { data, timestamp } => {
                tracing::info!("amf0 data, timestamp: {timestamp}, data: {data:?}");
            }
        }

        Ok(())
    }

    async fn on_publish(&mut self, stream_id: u32, app_name: &str, stream_name: &str) -> Result<(), ServerSessionError> {
        tracing::info!("publish, stream_id: {stream_id}, app_name: {app_name}, stream_name: {stream_name}");
        Ok(())
    }

    async fn on_unpublish(&mut self, stream_id: u32) -> Result<(), ServerSessionError> {
        tracing::info!("unpublish, stream_id: {stream_id}");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let listener = TcpListener::bind("[::]:1935").await.unwrap();
    tracing::info!("listening on [::]:1935");

    while let Ok((stream, addr)) = listener.accept().await {
        tracing::info!("accepted connection from {addr}");

        let session = ServerSession::new(stream, Handler);

        tokio::spawn(async move {
            if let Err(err) = session.run().instrument(tracing::info_span!("session", addr = %addr)).await {
                tracing::error!("session error: {:?}", err);
            }
        });
    }
}
