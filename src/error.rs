use serde::Serialize;
use thiserror::Error;
use warp::{
    http::{header, Response},
    hyper::StatusCode,
    reject::{Reject, Rejection},
    reply::Reply,
};

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not establish database connection: {0}")]
    DatabaseConnectionError(String),
    #[error("There has been an error executing a query: '{0}'")]
    QueryError(diesel::result::Error),
    #[error("There has been an error executing a serenity request: '{0}'")]
    SerenityError(serenity::Error),
    #[error("Failed to serialise data: {0}")]
    SerialisationError(String),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::DatabaseConnectionError(_)
            | Self::QueryError(_)
            | Self::SerenityError(_)
            | Self::SerialisationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> u32 {
        match self {
            Self::DatabaseConnectionError(_) => 500_001,
            Self::QueryError(_) => 500_002,
            Self::SerenityError(_) => 500_003,
            Self::SerialisationError(_) => 500_004,
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::QueryError(e)
    }
}

impl From<serenity::Error> for Error {
    fn from(e: serenity::Error) -> Self {
        Self::SerenityError(e)
    }
}

impl Reject for Error {}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    message: String,
    status: String,
    error_code: u32,
}

/// Creates a Rejection response for the given error and logs internal server errors.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(e) = err.find::<Error>() {
        let status_code = e.status_code();
        let message = e.to_string();
        let error_code = e.error_code();

        if let StatusCode::INTERNAL_SERVER_ERROR = status_code {
            log::error!("Encountered internal server error: {}", e);
        }

        let err_response = ErrorResponse {
            message,
            status: status_code.to_string(),
            error_code,
        };

        let response_builder = Response::builder()
            .status(status_code)
            .header(header::CONTENT_TYPE, "application/json");
        let response = response_builder
            .body(
                serde_json::to_vec(&err_response)
                    .map_err(|e| Error::SerialisationError(e.to_string()))?,
            )
            .map_err(|e| Error::SerialisationError(e.to_string()))?;

        Ok(response)
    } else {
        Err(err)
    }
}
