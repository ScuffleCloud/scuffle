use db_types::models::{Stream, StreamId};
use db_types::schema::streams;
use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{OptionExt, RequestExt, ResultExt};
use petname::Generator;
use tonic_types::ErrorDetails;

use crate::services::VideoApiSvc;

#[tonic::async_trait]
impl<G: video_api_traits::Global> pb::scufflecloud::video::api::v1::stream_service_server::StreamService for VideoApiSvc<G> {
    async fn create(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamCreateRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamCreateResponse>, tonic::Status> {
        let global = req.global::<G>()?;

        let payload = req.into_inner();
        let project_id = payload
            .project_id
            .parse()
            .into_tonic_err_with_field_violation("project_id", "invalid ID")?;

        // TODO: check permissions and if project exists

        let name = payload
            .name
            .or_else(|| petname::Petnames::large().generate_one(3, "-"))
            .into_tonic_internal_err("failed to generate random stream name")?;

        let stream = Stream {
            id: StreamId::new(),
            project_id,
            name,
        };

        let mut conn = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        diesel::insert_into(streams::dsl::streams)
            .values(&stream)
            .execute(&mut conn)
            .await
            .into_tonic_internal_err("failed to insert stream into database")?;

        Ok(tonic::Response::new(pb::scufflecloud::video::api::v1::StreamCreateResponse {
            stream: Some(stream.into()),
        }))
    }

    async fn get(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamGetRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamGetResponse>, tonic::Status> {
        let global = req.global::<G>()?;
        let payload = req.into_inner();
        let stream_id = payload.id.parse().into_tonic_err_with_field_violation("id", "invalid ID")?;

        // TODO: check permissions

        let stream = global
            .stream_loader()
            .load(stream_id)
            .await
            .ok()
            .into_tonic_internal_err("failed to load stream")?
            .into_tonic_err(tonic::Code::NotFound, "stream not found", ErrorDetails::new())?;

        Ok(tonic::Response::new(pb::scufflecloud::video::api::v1::StreamGetResponse {
            stream: Some(stream.into()),
        }))
    }

    async fn update(
        &self,
        _req: tonic::Request<pb::scufflecloud::video::api::v1::StreamUpdateRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamUpdateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn delete(
        &self,
        req: tonic::Request<pb::scufflecloud::video::api::v1::StreamDeleteRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamDeleteResponse>, tonic::Status> {
        let global = req.global::<G>()?;

        let payload = req.into_inner();
        let stream_id: StreamId = payload.id.parse().into_tonic_err_with_field_violation("id", "invalid ID")?;

        // TODO: check permissions

        let mut conn = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let stream = diesel::delete(streams::dsl::streams)
            .filter(streams::dsl::id.eq(stream_id))
            .returning(Stream::as_returning())
            .get_result::<Stream>(&mut conn)
            .await
            .into_tonic_internal_err("failed to insert stream into database")?;

        Ok(tonic::Response::new(pb::scufflecloud::video::api::v1::StreamDeleteResponse {
            stream: Some(stream.into()),
        }))
    }

    async fn list(
        &self,
        _req: tonic::Request<pb::scufflecloud::video::api::v1::StreamListRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::video::api::v1::StreamListResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
