use std::convert::Infallible;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::response::IntoResponse;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::{TcpListener, TcpStream};
#[cfg(not(feature = "runtime"))]
use tokio::spawn;
use tokio_rustls::TlsAcceptor;
use tracing::Instrument;

use super::{Backend, IncomingConnection, MakeService, ServiceHandler, SocketKind};
use crate::context::ContextFutExt;
#[cfg(feature = "runtime")]
use crate::runtime::spawn;
#[cfg(feature = "opentelemetry")]
use crate::telemetry::opentelemetry::OpenTelemetrySpanExt;

pub struct TlsBackend {
	listener: TcpListener,
	acceptor: Arc<TlsAcceptor>,
	builder: Arc<Builder<TokioExecutor>>,
	handler: crate::context::Handler,
}

impl TlsBackend {
	pub fn new(
		listener: TcpListener,
		acceptor: Arc<TlsAcceptor>,
		builder: Arc<Builder<TokioExecutor>>,
		ctx: &crate::context::Context,
	) -> Self {
		Self {
			listener,
			acceptor,
			builder,
			handler: ctx.new_child().1,
		}
	}
}

struct IncomingTlsConnection<'a> {
	remote_addr: std::net::SocketAddr,
	local_addr: std::net::SocketAddr,
	connection: &'a TcpStream,
}

impl IncomingConnection for IncomingTlsConnection<'_> {
	fn socket_kind(&self) -> SocketKind {
		SocketKind::TlsTcp
	}

	fn remote_addr(&self) -> std::net::SocketAddr {
		self.remote_addr
	}

	fn local_addr(&self) -> Option<std::net::SocketAddr> {
		Some(self.local_addr)
	}

	fn downcast<T: 'static>(&self) -> Option<&T> {
		if std::any::TypeId::of::<T>() == std::any::TypeId::of::<TcpStream>() {
			// Safety: We know that the type is TcpStream because we checked the type id.
			// We also know that the reference is valid because it is a reference to a field
			// of self.
			Some(unsafe { &*(self.connection as *const TcpStream as *const T) })
		} else {
			None
		}
	}
}

impl Backend for TlsBackend {
	async fn serve(self, make_service: impl MakeService) -> Result<crate::context::Handler, crate::http::server::Error> {
		tracing::debug!("listening for incoming connections on {:?}", self.listener.local_addr()?);
		loop {
			let ctx = self.handler.context();

			tracing::trace!("waiting for incoming connection");

			let Some((connection, addr)) = self.listener.accept().with_context(&ctx).await.transpose()? else {
				break;
			};

			let span = tracing::trace_span!("connection", remote_addr = %addr);
			let _guard = span.enter();

			tracing::trace!("accepted connection");

			let Some(service) = make_service
				.make_service(&IncomingTlsConnection {
					remote_addr: addr,
					local_addr: self.listener.local_addr()?,
					connection: &connection,
				})
				.await
			else {
				tracing::trace!("no service for connection, closing");
				continue;
			};

			tracing::trace!("spawning connection handler");

			spawn(
				Connection {
					connection,
					builder: self.builder.clone(),
					acceptor: self.acceptor.clone(),
					service,
					parent_ctx: ctx,
				}
				.serve()
				.in_current_span(),
			);
		}

		Ok(self.handler)
	}

	fn handler(&self) -> &crate::context::Handler {
		&self.handler
	}
}

struct Connection<S: ServiceHandler> {
	connection: TcpStream,
	builder: Arc<Builder<TokioExecutor>>,
	acceptor: Arc<TlsAcceptor>,
	service: S,
	parent_ctx: crate::context::Context,
}

impl<S: ServiceHandler> Connection<S> {
	async fn serve(self) {
		tracing::trace!("connection handler started");
		let connection = match self.acceptor.accept(self.connection).await {
			Ok(connection) => connection,
			Err(err) => {
				tracing::debug!(err = %err, "error accepting connection");
				self.service.on_error(err.into()).await;
				self.service.on_close().await;
				return;
			}
		};

		self.service.on_ready().await;
		#[cfg(feature = "opentelemetry")]
		tracing::Span::current().make_root();
		tracing::trace!("connection ready");

		let (_, handle) = self.parent_ctx.new_child();

		let make_ctx = {
			let handle = handle.clone();
			Arc::new(move || handle.context())
		};

		let service_fn = {
			let service = self.service.clone();
			let make_ctx = make_ctx.clone();
			let span = tracing::Span::current();

			service_fn(move |mut req: Request<Incoming>| {
				let service = service.clone();
				let make_ctx = make_ctx.clone();
				async move {
					let ctx = make_ctx();
					req.extensions_mut().insert(ctx.clone());
					let resp = service.on_request(req.map(Body::new)).await.into_response();
					drop(ctx);
					Ok::<_, Infallible>(resp)
				}.instrument(span.clone())
			})
		};

		let r = tokio::select! {
			r = self.builder.serve_connection_with_upgrades(TokioIo::new(connection), service_fn) => r,
			_ = async {
				self.parent_ctx.done().await;
				handle.shutdown().await;
			} => {
				Ok(())
			}
		};

		if let Err(err) = r {
			self.service.on_error(err.into()).await;
		}
		
		self.service.on_close().await;
		tracing::trace!("connection closed");
	}
}