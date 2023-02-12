use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use std::num::TryFromIntError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("axum http error: {0}")]
    Axum(#[from] axum::http::Error),
    #[error("not allowed to delete")]
    Delete,
    #[error("compression error: {0}")]
    Compression(String),
    #[error("entry not found")]
    NotFound,
    #[error("sqlite error: {0}")]
    Sqlite(rusqlite::Error),
    #[error("migrations error: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("wrong size")]
    WrongSize,
    #[error("illegal characters")]
    IllegalCharacters,
    #[error("integer conversion error: {0}")]
    IntConversion(#[from] TryFromIntError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("syntax highlighting error: {0}")]
    SyntaxHighlighting(#[from] syntect::Error),
    #[error("syntax parsing error: {0}")]
    SyntaxParsing(#[from] syntect::parsing::ParsingError),
    #[error("could not parse cookie: {0}")]
    CookieParsing(String),
}

#[derive(Serialize)]
pub struct JsonError {
    pub message: String,
}

/// Response carrying a status code and the error message as JSON.
pub type JsonErrorResponse = (StatusCode, Json<JsonError>);

impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::IllegalCharacters | Error::WrongSize | Error::CookieParsing(_) => {
                StatusCode::BAD_REQUEST
            }
            Error::Join(_)
            | Error::Compression(_)
            | Error::IntConversion(_)
            | Error::Migration(_)
            | Error::Sqlite(_)
            | Error::SyntaxHighlighting(_)
            | Error::SyntaxParsing(_)
            | Error::Axum(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Delete => StatusCode::FORBIDDEN,
        }
    }
}

impl From<Error> for JsonErrorResponse {
    fn from(err: Error) -> Self {
        let payload = Json::from(JsonError {
            message: err.to_string(),
        });

        (err.into(), payload)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound,
            _ => Error::Sqlite(err),
        }
    }
}
