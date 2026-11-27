use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::common::error::AppError;

pub struct AppJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(val)) => Ok(AppJson(val)),
            Err(rejection) => {
                let msg = match rejection {
                    JsonRejection::JsonDataError(e) => format!("Invalid request payload: {e}"),
                    JsonRejection::JsonSyntaxError(e) => format!("Malformed JSON syntax: {e}"),
                    JsonRejection::MissingJsonContentType(_) => {
                        "Expected request with `Content-Type: application/json`".to_string()
                    }
                    _ => "Failed to parse JSON body".to_string(),
                };
                Err(AppError::BadRequest(msg))
            }
        }
    }
}
