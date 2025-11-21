use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

#[derive(Clone, Debug)]
pub(crate) enum Authentication {
    Internal,
    External,
}

pub(crate) async fn auth<G: video_api_traits::Global>(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let tls_client_identity: Option<&scuffle_http::extensions::ClientIdentity> = req.extensions().get();

    if tls_client_identity.is_some() {
        req.extensions_mut()
            .insert(Authentication::Internal);
    } else {
        req.extensions_mut()
            .insert(Authentication::External);
    }

    Ok(next.run(req).await)
}
