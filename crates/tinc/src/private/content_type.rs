pub fn parse(headers: &http::HeaderMap) -> Result<Option<mediatype::MediaType>, ()> {
    let Some(content_type) = headers.get(http::header::CONTENT_TYPE) else {
        return Ok(None);
    };

    let content_type = content_type.to_str().map_err(|_| ())?;

    let content_type = mediatype::MediaType::parse(content_type).map_err(|_| ())?;

    Ok(Some(content_type))
}
