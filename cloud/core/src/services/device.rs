use core_pb::v1::device_service_server::DeviceService;
use core_pb::v1::{DeviceFinalizeRequest, DeviceFinalizeResponse, DeviceSetupRequest, DeviceSetupResponse};
use tonic::async_trait;


#[async_trait]
impl<G: core_traits::Global> DeviceService for crate::services::CoreSvc<G> {
    async fn setup(
        &self,
        request: tonic::Request<DeviceSetupRequest>,
    ) -> tonic::Result<tonic::Response<DeviceSetupResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn finalize(
        &self,
        request: tonic::Request<DeviceFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<DeviceFinalizeResponse>> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

