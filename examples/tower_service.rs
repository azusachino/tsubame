use std::{pin::Pin, task::Poll};

use http::{Request, Response, StatusCode};
use tower::Service;

struct Demo;

impl Service<Request<Vec<u8>>> for Demo {
    type Response = Response<Vec<u8>>;
    type Error = http::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Request<Vec<u8>>) -> Self::Future {
        // create the body
        let body: Vec<u8> = "hello, world\n".as_bytes().to_owned();

        // http response
        let res = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("fail to create response");

        let fut = async { Ok(res) };

        // Return the response as an immediate future
        Box::pin(fut)
    }
}

fn main() {}
