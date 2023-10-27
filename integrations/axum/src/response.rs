use axum::{
    body::{boxed, Body, BoxBody},
    http,
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use axum::body::Full;

/// Responder for a GraphQL response.
///
/// This contains a batch response, but since regular responses are a type of
/// batch response it works for both.
pub struct GraphQLResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp.into())
    }
}

impl From<async_graphql::BatchResponse> for GraphQLResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        Self(resp)
    }
}

impl IntoResponse for GraphQLResponse {
    fn into_response(self) -> Response<BoxBody> {
        // let body: Body = serde_json::to_string(&self.0).unwrap().into();
        let body = Full::new(bincode::serialize(&self.0).unwrap().into());
        let mut resp = Response::new(boxed(body));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                if let Ok(value) = HeaderValue::from_str(&cache_control) {
                    resp.headers_mut()
                        .insert(http::header::CACHE_CONTROL, value);
                }
            }
        }

        resp.headers_mut().extend(self.0.http_headers());
        resp
    }
}
