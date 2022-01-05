use std::sync::Arc;

use tera::Tera;

use crate::errors::AppError;
use crate::upload::UploadManager;

pub struct AppData {
    templates: Tera,
    manager: Arc<Box<UploadManager>>,
}

impl AppData {
    pub fn new() -> Result<Self, AppError> {
        let upload_directory = std::env::var("FS_UPLOAD_DIR")
            .ok()
            .unwrap_or("./tmp".to_string());

        debug!("Upload directory: {}", upload_directory);

        let templates_directory = std::env::var("FS_TEMPLATES")
            .ok()
            .unwrap_or("templates".to_string());

        debug!("Templates directory: {}", templates_directory);

        let mut templates = Tera::new(&format!("{}/**/*", templates_directory))?;
        templates.full_reload()?;

        let manager = UploadManager::new(std::path::PathBuf::from(upload_directory))?;

        Ok(AppData {
            templates,
            manager: Arc::new(Box::new(manager)),
        })
    }

    pub fn templates(&self) -> &Tera {
        &self.templates
    }

    pub fn manager(&self) -> Arc<Box<UploadManager>> {
        self.manager.clone()
    }
}
