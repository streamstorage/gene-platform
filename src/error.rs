use actix_web::{
    HttpResponse, error,
    http::{StatusCode, header::ContentType},
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display("{error_message}")]
    BadRequest { error_message: String },

    #[display("{error_message}")]
    Unauthorized { error_message: String },

    #[display("{error_message}")]
    Forbidden { error_message: String },

    #[display("{error_message}")]
    NotFound { error_message: String },

    #[display("{error_message}")]
    InternalServerError { error_message: String },
}

impl error::ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ServiceError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ServiceError::NotFound { .. } => StatusCode::NOT_FOUND,
            ServiceError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self.to_string())
    }
}

impl ServiceError {
    pub fn internal_error<T>(e: T) -> Self
    where
        T: ToString,
    {
        ServiceError::InternalServerError {
            error_message: e.to_string(),
        }
    }

    pub fn internal_error_msg<T>(e: T, err: &str) -> Self
    where
        T: ToString,
    {
        ServiceError::InternalServerError {
            error_message: format!("{}, err: {}", err, e.to_string()),
        }
    }

    pub fn pool_error<T>(e: T) -> ServiceError
    where
        T: ToString,
    {
        Self::internal_error_msg(e, "db connection failed")
    }

    pub fn bad_request(err: &str) -> Self {
        ServiceError::BadRequest {
            error_message: err.to_string(),
        }
    }

    pub fn unauthorized(err: &str) -> Self {
        ServiceError::Unauthorized {
            error_message: err.to_string(),
        }
    }

    pub fn forbidden(err: &str) -> Self {
        ServiceError::Forbidden {
            error_message: err.to_string(),
        }
    }
}
