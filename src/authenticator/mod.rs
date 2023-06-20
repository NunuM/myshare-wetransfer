use std::sync::Arc;

use crate::app_configs::AuthStrategy;
#[cfg(target_os = "linux")]
use crate::authenticator::pam::PamAuthenticator;
use crate::authenticator::passwd::PasswdFileAuthenticator;
use crate::errors::AppError;

#[cfg(target_os = "linux")]
mod pam;
mod passwd;

pub trait Authenticator: Sync + Send {
    fn authenticate(&self, username: &str, password: &str) -> bool;
}


pub fn get_authenticator(auth_strategy: &AuthStrategy) -> Result<Arc<Box<dyn Authenticator>>, AppError> {
    Ok(match auth_strategy {
        AuthStrategy::File(file) => {
            Arc::new(Box::new(PasswdFileAuthenticator::new(file)?))
        }
        #[cfg(target_os = "linux")]
        AuthStrategy::PamModule(pam_module) => {
            Arc::new(Box::new(PamAuthenticator::new(pam_module.to_string())))
        }
    })
}
