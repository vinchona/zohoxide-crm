use crate::response::ApiErrorResponse;
use thiserror::Error;

/// Various errors returned by the API.
#[derive(Error, Debug)]
pub enum ClientError {
    /// General error message that encompasses almost any non-token related error message.
    #[error(transparent)]
    General(#[from] anyhow::Error),

    /// Error returned when a response from the API does not deserialize into the user's
    /// custom data type. The raw response will be returned with this error.
    #[error("{0}")]
    UnexpectedResponseType(String),

    /// Error return when a response from the API is empty
    #[error("Empty response")]
    EmptyResponse,

    /// Error returned from most API requests.
    #[error("{0}")]
    ApiError(ApiErrorResponse),
}
