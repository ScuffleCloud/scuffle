use http_body_util::BodyExt;
use tinc::TincService;
use tower::Service;

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("simple_service");
}

struct Svc {}

#[tonic::async_trait]
impl pb::simple_service_server::SimpleService for Svc {
    async fn ping(&self, request: tonic::Request<pb::PingRequest>) -> tonic::Result<tonic::Response<pb::PingResponse>> {
        Ok(pb::PingResponse {
            result: format!("{} - pong", request.get_ref().arg),
        }
        .into())
    }
}

#[tokio::test]
async fn test_simple_service_grpc() {
    let mut client =
        pb::simple_service_client::SimpleServiceClient::new(pb::simple_service_server::SimpleServiceServer::new(Svc {}));

    let response = client.ping(pb::PingRequest { arg: "grpc".into() }).await.unwrap();

    assert_eq!(response.get_ref().result, "grpc - pong");
}

#[tokio::test]
async fn test_simple_service_rest_post() {
    let mut client = pb::simple_service_tinc::SimpleServiceTinc::new(Svc {}).into_router();

    let req = http::Request::builder()
        .uri("/ping")
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

    assert_eq!(response["result"], "http - pong");
}

#[tokio::test]
async fn test_simple_service_rest_get() {
    let mut client = pb::simple_service_tinc::SimpleServiceTinc::new(Svc {}).into_router();

    let req = http::Request::builder()
        .uri("/ping/http_get")
        .method("GET")
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        .unwrap();

    let resp = client.call(req).await.unwrap();

    assert_eq!(
        resp.headers().get(http::header::CONTENT_TYPE).map(|h| h.as_bytes()),
        Some(b"application/json" as &[u8])
    );

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["result"], "http_get - pong");
}

#[test]
fn test_simple_service_rest_schema() {
    let svc = pb::simple_service_tinc::SimpleServiceTinc::new(Svc {});

    insta::assert_json_snapshot!(svc.openapi_schema());
}
