use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpResponse;
use futures::future::{ok, Either, Ready};
use futures::task::{Context, Poll};

pub struct BasicAuth;

impl<S, B> Transform<S> for BasicAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = BasicAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(BasicAuthMiddleware { service })
    }
}

pub struct BasicAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for BasicAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let user_password = std::env::var("FS_AUTH")
            .ok()
            .unwrap_or("cm9vdDpyb290".to_string());

        debug!("Base64 authentication: {}", user_password);

        let is_auth = req
            .headers()
            .get("Authorization")
            .map(|h| h.to_str().ok())
            .flatten()
            .and_then(|value| Some(value.eq(&format!("Basic {}", user_password))));

        if let Some(true) = is_auth {
            Either::Left(self.service.call(req))
        } else {
            Either::Right(ok(req.into_response(
                HttpResponse::Unauthorized()
                    .set_header(
                        "WWW-Authenticate",
                        "Basic realm=\"User Visible Realm\", charset=\"UTF-8\"",
                    )
                    .finish()
                    .into_body(),
            )))
        }
    }
}
