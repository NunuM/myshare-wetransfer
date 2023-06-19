use std::collections::HashMap;
use crate::authenticator::Authenticator;
use crate::errors::AppError;

pub struct PasswdFileAuthenticator {
    users: HashMap<String, String>,
}

impl PasswdFileAuthenticator {
    pub fn new(file: &std::path::PathBuf) -> Result<Self, AppError> {
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