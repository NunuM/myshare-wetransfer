use std::time::SystemTimeError;

use actix_multipart::MultipartError;
use actix_web::rt::blocking::BlockingError;
use actix_web::HttpResponse;

use zip::result::ZipError;

#[derive(Debug)]
pub enum AppError {
    InitError(String),
    FileSystemError(String),
    ArchiveError(String),
    UploadFailed(String),
    ThreadError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InitError(ref reason) => write!(f, "init error {}", reason),
            AppError::FileSystemError(ref reason) => write!(f, "fs error {}", reason),
            AppError::ArchiveError(ref reason) => write!(f, "zip error {}", reason),
            AppError::UploadFailed(ref reason) => write!(f, "upload error {}", reason),
            AppError::ThreadError(ref reason) => write!(f, "thread error {}", reason),
        }
    }
}

impl From<actix_multipart::MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::UploadFailed(format!("{:?}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystemError(err.to_string())
    }
}

impl<E: std::fmt::Debug + Into<AppError>> From<actix_web::error::BlockingError<E>> for AppError {
    fn from(e: BlockingError<E>) -> Self {
        match e {
            BlockingError::Error(e) => e.into(),
            BlockingError::Canceled => AppError::ThreadError(format!("Thread canceled")),
        }
    }
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        error!("Some error: {:?}", self);

        match self {
            AppError::UploadFailed(ref reason) => HttpResponse::BadRequest()
                .set_header("Content-Type", "text/plain")
                .body(reason),
            AppError::ArchiveError(ref reason) => HttpResponse::BadRequest()
                .set_header("Content-Type", "text/plain")
                .body(reason),
            AppError::FileSystemError(ref reason) => HttpResponse::InternalServerError()
                .set_header("Content-Type", "text/plain")
                .body(reason),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: ZipError) -> Self {
        AppError::UploadFailed(format!("{:?}", err))
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(err: SystemTimeError) -> Self {
        AppError::ArchiveError(format!("{:?}", err))
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        AppError::InitError(err.to_string())
    }
}
