use std::sync::{Arc, Mutex};

use hyper::header::IntoHeaderName;
use hyper::Request;
use tonic::async_trait;

use crate::http::router::ext::RequestExt;

#[derive(Clone, Default)]
struct ResponseHeadersMagic(Arc<Mutex<hyper::HeaderMap>>);

pub struct ResponseHeadersMiddleware;

impl ResponseHeadersMiddleware {
	pub fn new() -> Self {
		Self
	}
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static, E: Send + 'static> super::Middleware<I, O, E> for ResponseHeadersMiddleware {
	async fn handle(&self, mut req: Request<I>, next: super::NextFn<I, O, E>) -> Result<hyper::Response<O>, E> {
		let headers = ResponseHeadersMagic::default();
		req.provide(headers.clone());

		let mut resp = next(req).await?;

		let headers = headers.0.lock().expect("failed to lock headers");
		headers.iter().for_each(|(k, v)| {
			resp.headers_mut().insert(k, v.clone());
		});

		Ok(resp)
	}
}

pub trait ResponseHeadersRequestExt {
	fn set_response_header<K, V>(&self, key: K, value: V)
	where
		K: IntoHeaderName,
		V: Into<hyper::header::HeaderValue>;
}

impl<B> ResponseHeadersRequestExt for Request<B> {
	fn set_response_header<K, V>(&self, key: K, value: V)
	where
		K: IntoHeaderName,
		V: Into<hyper::header::HeaderValue>,
	{
		let headers = self.data::<ResponseHeadersMagic>().unwrap();
		let mut headers = headers.0.lock().expect("failed to lock headers");
		key.insert(&mut headers, value.into());
	}
}
