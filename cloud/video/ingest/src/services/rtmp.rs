use scuffle_rtmp::session::server::{ServerSessionError, SessionData, SessionHandler};

pub(crate) struct Handler;

impl SessionHandler for Handler {
    async fn on_data(&mut self, stream_id: u32, data: SessionData) -> Result<(), ServerSessionError> {
        // Handle incoming video/audio/meta data
        match data {
            SessionData::Amf0 { timestamp, data } => {
                tracing::info!(stream_id, timestamp, data_len = data.len(), "received AMF0 metadata");
            }
            SessionData::Audio { timestamp, data } => {
                tracing::info!(stream_id, timestamp, data_len = data.len(), "received audio data");
            }
            SessionData::Video { timestamp, data } => {
                tracing::info!(stream_id, timestamp, data_len = data.len(), "received video data");
            }
        }

        Ok(())
    }

    async fn on_publish(&mut self, stream_id: u32, app_name: &str, stream_name: &str) -> Result<(), ServerSessionError> {
        // Handle the publish event
        tracing::info!(stream_id, app_name, stream_name, "stream published");
        Ok(())
    }

    async fn on_unpublish(&mut self, stream_id: u32) -> Result<(), ServerSessionError> {
        // Handle the unpublish event
        tracing::info!(stream_id, "stream unpublished");
        Ok(())
    }
}
