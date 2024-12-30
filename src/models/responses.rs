use core::fmt;
use std::error::Error;

use reqwest::{Response, StatusCode};

#[derive(Debug)]
pub struct SimpleResponse {
    pub status: StatusCode,
    pub body: String,
}

impl fmt::Display for SimpleResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (status {})", self.body, self.status)
    }
}

#[derive(Debug)]
pub struct ErrorResponse {
    pub response: SimpleResponse,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.response)
    }
}

impl Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl SimpleResponse {
    async fn from_response(res: Response) -> Self {
        SimpleResponse {
            status: res.status(),
            body: res.text().await.unwrap(),
        }
    }
    // async fn from_error(err: reqwest::Error) -> Self {
    //     SimpleResponse {
    //         status: err.status().unwrap(),
    //         body: err.to_string(),
    //     }
    // }
}

pub async fn response_to_result(res: Response) -> Result<SimpleResponse, ErrorResponse> {
    let status_body = SimpleResponse::from_response(res).await;

    if status_body.status.is_success() {
        Ok(status_body)
    } else {
        Err(ErrorResponse {
            response: status_body,
        })
    }
}
