use std::io;

use rocket::Responder;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct InnerResponse {
    message: String,
    data: Option<Value>,
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct Success(pub String);

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
pub struct Error(pub String);

impl Success {
    pub fn empty() -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: "success".to_string(),
                                        data: None,
        }).unwrap())
    }

    pub fn data(data: Value) -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: "success".to_string(),
                                        data: Some(data),
        }).unwrap())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: value.to_string(),
                                        data: None,
        }).unwrap())
    }
}
