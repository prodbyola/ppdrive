use std::{env::VarError, fmt::Display, string::FromUtf8Error};

use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum AppError {
    InitError(String),
    InternalServerError(String),
    DatabaseError(String),
    AuthorizationError(String),
    ParsingError(String),
    IOError(String),
    NotImplemented(String),
    NotFound(String),
    PermissionDenied(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InitError(msg) => write!(f, "{msg}"),
            AppError::InternalServerError(msg) => write!(f, "{msg}"),
            AppError::DatabaseError(msg) => write!(f, "{msg}"),
            AppError::AuthorizationError(msg) => write!(f, "{msg}"),
            AppError::ParsingError(msg) => write!(f, "{msg}"),
            AppError::IOError(msg) => write!(f, "{msg}"),
            AppError::NotImplemented(msg) => write!(f, "{msg}"),
            AppError::NotFound(msg) => write!(f, "{msg}"),
            AppError::PermissionDenied(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<VarError> for AppError {
    fn from(value: VarError) -> Self {
        AppError::InternalServerError(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::InternalServerError(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IOError(value.to_string())
    }
}

impl From<MultipartError> for AppError {
    fn from(value: MultipartError) -> Self {
        AppError::InternalServerError(value.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        use sqlx::Error::*;
        match value {
            RowNotFound => {
                AppError::NotFound("error getting the requested resource from database".to_string())
            }
            _ => AppError::DatabaseError(value.to_string()),
        }
    }
}

impl From<chacha20poly1305::aead::Error> for AppError {
    fn from(value: chacha20poly1305::aead::Error) -> Self {
        AppError::InternalServerError(value.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let resp = match self {
            AppError::AuthorizationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::PermissionDenied(msg) => (StatusCode::FORBIDDEN, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        resp.into_response()
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(value: FromUtf8Error) -> Self {
        AppError::InitError(value.to_string())
    }
}
