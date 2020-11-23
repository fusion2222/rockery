use hyper::{Body, Response, http::StatusCode};


pub struct HTTPResponse {
    pub status_code: StatusCode,
    pub body: String, // TODO: Optimize to &str if possible 
}

impl HTTPResponse {
    pub fn as_hyper_response(&self) -> Response<Body>{
        Response::builder()
            .status(self.status_code)
            .body(Body::from(self.body.clone()))
            .unwrap()
    }
}
