use std::collections::HashMap;
use std::sync::Arc;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpResponse;
use base64::Engine;
use futures::future::{ok, Either, Ready};
use futures::task::{Context, Poll};
use crate::app_configs::AuthStrategy;
use crate::errors::AppError;

#[derive(Clone)]
pub struct BasicAuth {
    authenticator: Arc<Box<dyn Authenticator>>,
}

impl BasicAuth {
    pub fn new(auth_strategy: &AuthStrategy) -> Result<Self, AppError> {
        Ok(match auth_strategy {
            AuthStrategy::File(file) => {
                BasicAuth {
                    authenticator: Arc::new(Box::new(PasswdFileAuthenticator::new(file)?))
                }
            }
            AuthStrategy::PamModule(pam_module) => {
                BasicAuth {
                    authenticator: Arc::new(Box::new(PamAuthenticator {
                        pam_module: pam_module.to_string()
                    }))
                }
            }
        })
    }
}

impl<S, B> Transform<S> for BasicAuth
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = BasicAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(BasicAuthMiddleware { service, authenticator: self.authenticator.clone() })
    }
}

pub struct BasicAuthMiddleware<S> {
    service: S,
    authenticator: Arc<Box<dyn Authenticator>>,
}

impl<S> BasicAuthMiddleware<S> {
    fn authenticate(&self, username: &str, password: &str) -> bool {
        self.authenticator.authenticate(username, password)
    }
}

impl<S, B> Service for BasicAuthMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
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

        let is_auth = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .map(|h| h.to_str().ok())
            .flatten()
            .map(|value| {
                value
                    .strip_prefix("Basic ")
                    .map(|v| {
                        base64::engine::general_purpose::STANDARD.decode(v)
                            .map(|values| String::from_utf8(values).unwrap_or(String::new()))
                            .map(|credentials| match credentials.split_once(":") {
                                Some((user, password)) => self.authenticate(user, password),
                                _ => false
                            })
                            .unwrap_or(false)
                    })
            })
            .flatten();

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

trait Authenticator: Sync + Send {
    fn authenticate(&self, username: &str, password: &str) -> bool;
}


struct PamAuthenticator {
    pam_module: String,
}

impl Clone for PamAuthenticator {
    fn clone(&self) -> Self {
        PamAuthenticator {
            pam_module: self.pam_module.clone()
        }
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> bool {
        pam::Authenticator::with_password(&self.pam_module)
            .map(|mut auth| {
                auth.get_handler().set_credentials(username, password);

                auth.authenticate().is_ok()
            })
            .unwrap_or(false)
    }
}

struct PasswdFileAuthenticator {
    users: HashMap<String, String>,
}

impl PasswdFileAuthenticator {
    fn new(file: &std::path::PathBuf) -> Result<Self, AppError> {
        let mut users = HashMap::new();

        let file_content = std::fs::read_to_string(file)
            .map_err(|e| AppError::InitError(format!("Failed to read password file:{}", e)))?;

        let lines = file_content.lines();

        lines
            .filter(|l| !l.starts_with("#"))
            .for_each(|line| {
                let some_parts = line.split_once(" ");

                if let Some((user, pass)) = some_parts {
                    users.insert(user.to_string(), pass.to_string());
                }
            });

        Ok(PasswdFileAuthenticator {
            users
        })
    }
}

impl Clone for PasswdFileAuthenticator {
    fn clone(&self) -> Self {
        PasswdFileAuthenticator {
            users: self.users.clone()
        }
    }
}

impl Authenticator for PasswdFileAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> bool {
        self.users.get(username).map(|pass| pass.eq(password)).unwrap_or(false)
    }
}