use crate::{define::IncomingTranscoder, global::GlobalState};
use std::{
    pin::Pin,
    sync::{Arc, Weak},
    time::Duration,
};

use async_stream::try_stream;
use common::prelude::FutureTimeout;
use futures_util::Stream;
use tonic::{async_trait, Request, Response, Status, Streaming};

use pb::{
    ext::UlidExt,
    scuffle::video::internal::{
        ingest_server, ingest_watch_request, IngestWatchRequest, IngestWatchResponse,
    },
};

pub struct IngestServer {
    global: Weak<GlobalState>,
}

impl IngestServer {
    pub fn new(global: &Arc<GlobalState>) -> ingest_server::IngestServer<Self> {
        ingest_server::IngestServer::new(Self {
            global: Arc::downgrade(global),
        })
    }
}

type Result<T> = std::result::Result<T, Status>;

#[async_trait]
impl ingest_server::Ingest for IngestServer {
    /// Server streaming response type for the Watch method.
    type WatchStream = Pin<Box<dyn Stream<Item = Result<IngestWatchResponse>> + Send + Sync>>;

    async fn watch(
        &self,
        request: Request<Streaming<IngestWatchRequest>>,
    ) -> Result<Response<Self::WatchStream>> {
        let global = self.global.upgrade().ok_or_else(|| {
            Status::internal("Global state was dropped, cannot handle ingest request")
        })?;

        let mut request = request.into_inner();

        let Some(message) = request.message().await? else {
            return Err(Status::invalid_argument("No message provided"));
        };

        let open_req = match &message.message {
            Some(ingest_watch_request::Message::Open(message)) => message,
            Some(_) => return Err(Status::invalid_argument("Invalid message type")),
            None => return Err(Status::invalid_argument("No message provided")),
        };

        let ulid = open_req.request_id.to_ulid();

        let Some(handler) = global.requests.lock().await.remove(&ulid) else {
            return Err(Status::not_found("No ingest request found with that UUID"));
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);

        handler
            .send(IncomingTranscoder {
                ulid,
                message,
                streaming: request,
                transcoder: tx,
            })
            .await
            .map_err(|_| Status::internal("Failed to send request to handler"))?;

        let Ok(Some(message)) = rx.recv().timeout(Duration::from_secs(1)).await else {
            return Err(Status::internal("Failed to receive response from handler"));
        };

        let output = try_stream!({
            yield message;

            while let Some(message) = rx.recv().await {
                yield message;
            }
        });

        Ok(Response::new(Box::pin(output)))
    }
}
