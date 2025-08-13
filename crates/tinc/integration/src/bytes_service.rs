use http_body_util::BodyExt;
use rand::{RngCore, SeedableRng};
use tinc::TincService;
use tower::Service;

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("bytes_service");
}

struct Svc {}

#[tonic::async_trait]
impl pb::bytes_service_server::BytesService for Svc {
    async fn bytes(&self, request: tonic::Request<pb::BytesPayload>) -> tonic::Result<tonic::Response<pb::BytesPayload>> {
        Ok(request.into_inner().into())
    }
}

#[tokio::test]
async fn test_bytes_service_grpc() {
    let mut client =
        pb::bytes_service_client::BytesServiceClient::new(pb::bytes_service_server::BytesServiceServer::new(Svc {}));

    let response = client
        .bytes(pb::BytesPayload {
            data: vec![0; 100],
            mime: "application/octet-stream".into(),
        })
        .await
        .unwrap();

    assert_eq!(response.get_ref().mime, "application/octet-stream");
    assert_eq!(response.get_ref().data, vec![0; 100]);
}

#[tokio::test]
async fn test_bytes_service_rest_post_json() {
    let mut client = pb::bytes_service_tinc::BytesServiceTinc::new(Svc {}).into_router();

    let req = http::Request::builder()
        .uri("/upload")
        .method("POST")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(http_body_util::Full::new(bytes::Bytes::from_static(
            r#"{ "arg": "http" }"#.as_bytes(),
        )))
        .unwrap();

    let resp = client.call(req).await.unwrap();

    assert_eq!(
        resp.headers().get(http::header::CONTENT_TYPE).map(|h| h.as_bytes()),
        Some(b"application/json" as &[u8])
    );

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["arg"], "http");
}

#[tokio::test]
async fn test_bytes_service_rest_post_binary() {
    let mut client = pb::bytes_service_tinc::BytesServiceTinc::new(Svc {}).into_router();

    let mut rand = rand::rngs::SmallRng::seed_from_u64(100);
    let mut random_data = vec![0; 1000];
    rand.fill_bytes(&mut random_data);

    let req = http::Request::builder()
        .uri("/upload")
        .method("POST")
        .header(http::header::CONTENT_TYPE, "some-random-content-type/xd")
        .body(http_body_util::Full::new(bytes::Bytes::from(random_data.clone())))
        .unwrap();

    let resp = client.call(req).await.unwrap();

    assert_eq!(
        resp.headers().get(http::header::CONTENT_TYPE).map(|h| h.as_bytes()),
        Some(b"some-random-content-type/xd" as &[u8])
    );

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body, random_data);
}

#[test]
fn test_bytes_service_rest_schema() {
    let svc = pb::bytes_service_tinc::BytesServiceTinc::new(Svc {});

    insta::assert_json_snapshot!(svc.openapi_schema());
}
