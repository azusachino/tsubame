use axum::{body::Body, http::Request, routing::any_service, Router};
use http::Response;
use std::{convert::Infallible, net::SocketAddr};
use tower::service_fn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route(
            "/",
            any_service(service_fn(|_: Request<Body>| async {
                let ret = Response::new(Body::from("Hello"));
                Ok::<_, Infallible>(ret)
            })),
        )
        .route_service(
            "/foo",
            service_fn(|req: Request<Body>| async move {
                let body = Body::from(format!("Hi from `{} /foo`", req.method()));
                let body = axum::body::boxed(body);
                let res = Response::new(body);
                Ok::<_, Infallible>(res)
            }),
        );

    let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
