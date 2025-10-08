use crate::services::VideoApiSvc;

#[tonic::async_trait]
impl<G: video_api_traits::Global> pb::scufflecloud::video::api::v1::stream_service_server::StreamService for VideoApiSvc<G> {
    async fn create(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamCreateRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamCreateResponse>, tonic::Status> {
        todo!()
    }

    async fn get(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamGetRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamGetResponse>, tonic::Status> {
        todo!()
    }

    async fn update(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamUpdateRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamUpdateResponse>, tonic::Status> {
        todo!()
    }

    async fn delete(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamDeleteRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamDeleteResponse>, tonic::Status> {
        todo!()
    }

    async fn list(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamListRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamListResponse>, tonic::Status> {
        todo!()
    }
}
