use std::sync::Arc;

use tera::Tera;

use crate::app_configs::ApplicationConfigurations;
use crate::errors::AppError;
use crate::upload::UploadManager;

pub struct AppData {
    templates: Tera,
    manager: Arc<Box<UploadManager>>,
}

impl AppData {
    pub fn new(configs: ApplicationConfigurations) -> Result<Self, AppError> {
        let upload_directory = configs.upload_configs().upload_directory();

        let templates_directory = configs.ui_configs().tera_templates();

        let mut templates = Tera::new(&format!("{}/**/*", templates_directory.display()))?;
        templates.full_reload()?;

        let manager = UploadManager::new(
            std::path::PathBuf::from(upload_directory),
            configs.upload_configs().max_size(),
        );

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
