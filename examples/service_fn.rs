use tower::{service_fn, BoxError, Service, ServiceExt};

struct Req;

struct Res(&'static str);

impl Req {
    fn new() -> Self {
        Self
    }
}

impl Res {
    fn new(body: &'static str) -> Self {
        Self(body)
    }

    fn into_body(self) -> &'static str {
        self.0
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {

    async fn handle(_: Req) -> Result<Res, BoxError> {
        let res = Res::new("Hello World");
        Ok(res)
    }

    let mut svc = service_fn(handle);

    let res = svc.ready().await?.call(Req::new()).await?;

    assert_eq!("Hello World", res.into_body());
    Ok(())
}
