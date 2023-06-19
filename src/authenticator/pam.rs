use crate::authenticator::Authenticator;

pub struct PamAuthenticator {
    pam_module: String,
}

impl PamAuthenticator {
    pub fn new(pam_module: String) -> Self {
        Self { pam_module }
    }
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