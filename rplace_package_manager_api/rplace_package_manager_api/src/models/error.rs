use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};

pub struct ErrorBuilder {
    pub message: String,
    pub err: String,
    pub status_code: StatusCode,
}
impl ErrorBuilder {
    pub fn builder(status_code: StatusCode) -> Self {
        Self {
            message: String::new(),
            err: String::new(),
            status_code: status_code,
        }
    }
    pub fn new(message: String, err: String, status_code: StatusCode) -> Self {
        Self {
            message: message,
            err: err,
            status_code,
        }
    }
    pub fn message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status_code = status;
        self
    }
    pub fn err<T: ToString>(mut self, err: T) -> Self {
        self.err = err.to_string();
        self
    }
    pub fn json(self) -> (StatusCode, Json<Value>) {
        let json = Json(json!({
                "message": self.message,
                "err": self.err
        }));
        return (self.status_code, json);
    }
}
