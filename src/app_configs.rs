use std::net::IpAddr;
use std::str::FromStr;
use ini::Ini;
use crate::errors::AppError;


const FSHARE_CONF_FILE_ENV_KEY: &'static str = "FSHARE_CONF_FILE";

const DEFAULT_NUMBER_OF_THREADS: u16 = 1;

const DEFAULT_MAX_SIZE: usize = 1000000000;

const COMMON_LOG_FORMAT: &'static str = "%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T \"%{X-Correlation-Id}i\"";

#[derive(Clone, Debug)]
pub struct UploadConfiguration {
    max_size: usize,
    upload_directory: std::path::PathBuf,
}

impl UploadConfiguration {
    fn section_name() -> &'static str {
        return "upload";
    }

    fn try_from(configs: &Ini) -> Result<Self, AppError> {
        let properties = configs
            .section(Some(Self::section_name()))
            .ok_or(AppError::InitError(format!("Section [{}] is missing from configuration file", Self::section_name())))?;

        let max_upload_size = properties.get("max_size")
            .and_then(|size_str| size_str.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_SIZE);

        let upload_directory = properties.get("upload_directory")
            .map(|upload| std::path::PathBuf::from(upload))
            .unwrap_or(std::env::temp_dir());

        if !upload_directory.exists() {
            std::fs::create_dir_all(&upload_directory)
                .map_err(|e| {
                    AppError::InitError(format!("Error creating upload directory: {}", e))
                })?
        } else {
            let read_only = upload_directory.metadata()
                .map(|metadata| metadata.permissions().readonly())
                .unwrap_or(true);


            if read_only {
                return Err(AppError::InitError(format!("Upload directory without write permissions")));
            }
        }

        Ok(UploadConfiguration {
            max_size: max_upload_size,
            upload_directory,
        })
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }
    pub fn upload_directory(&self) -> &std::path::PathBuf {
        &self.upload_directory
    }
}

#[derive(Clone, Debug)]
pub struct UiConfiguration {
    tera_templates: std::path::PathBuf,
}

impl UiConfiguration {
    fn section_name() -> &'static str {
        return "ui";
    }

    fn try_from(configs: &Ini) -> Result<Self, AppError> {
        let properties = configs
            .section(Some(Self::section_name()))
            .ok_or(AppError::InitError(format!("Section [{}] is missing from configuration file", Self::section_name())))?;


        let tera_directory = properties.get("tera_templates")
            .map(|dir| std::path::PathBuf::from(dir))
            .ok_or(AppError::InitError(format!("Missing tera_templates from section: [{}]", Self::section_name())))?;

        if !tera_directory.exists() {
            return Err(AppError::InitError(format!("Tera templates directory not exists")));
        }

        Ok(UiConfiguration {
            tera_templates: tera_directory
        })
    }

    pub fn tera_templates(&self) -> &std::path::PathBuf {
        &self.tera_templates
    }
}

#[derive(Clone, Debug)]
pub enum AuthStrategy {
    File(std::path::PathBuf),
    PamModule(String),
}

impl AuthStrategy {
    fn try_from(configs: &Ini) -> Result<Self, AppError> {
        let auth_strategy = configs
            .get_from(Some(ServerConfiguration::section_name()), "auth_strategy")
            .unwrap_or("auth_file");


        if auth_strategy.eq("auth_file") {
            let users_file = configs.get_from(Some(auth_strategy), "user_pass_file")
                .map(|path| std::path::Path::new(path))
                .unwrap_or(std::path::Path::new("users.txt"));

            if !users_file.exists() {
                return Err(AppError::InitError(format!("Unable to read users password file: {}", users_file.display())));
            }

            Ok(AuthStrategy::File(users_file.to_path_buf()))
        } else {

            let pam_module = configs.get_from(Some(auth_strategy), "pam_module_name")
                .ok_or(AppError::InitError(format!("Missing pam module name in section:[{}]", auth_strategy)))?;

            Ok(AuthStrategy::PamModule(pam_module.to_string()))
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServerConfiguration {
    host: IpAddr,
    port: u16,
    number_thread: u16,
    auth_strategy: AuthStrategy,
    log_format: String,
}

impl ServerConfiguration {
    fn section_name() -> &'static str {
        return "server";
    }


    fn try_from(configs: &Ini) -> Result<Self, AppError> {
        let properties = configs
            .section(Some(Self::section_name()))
            .ok_or(AppError::InitError(format!("Section [{}] is missing from configuration file", Self::section_name())))?;

        let host = properties.get("host")
            .and_then(|host_str| std::net::IpAddr::from_str(host_str).ok())
            .ok_or(AppError::InitError(format!("Missing valid host address from [{}] section", Self::section_name())))?;

        let port = properties.get("port")
            .and_then(|port_str| port_str.parse::<u16>().ok())
            .ok_or(AppError::InitError(format!("Missing valid port from [{}] section", Self::section_name())))?;

        let workers = properties.get("workers")
            .and_then(|workers_str| workers_str.parse::<u16>().ok())
            .unwrap_or(DEFAULT_NUMBER_OF_THREADS);

        let auth_strategy = AuthStrategy::try_from(configs)?;

        let log_format = properties.get("log_format")
            .unwrap_or(COMMON_LOG_FORMAT)
            .to_string();

        Ok(ServerConfiguration {
            host,
            port,
            number_thread: workers,
            auth_strategy,
            log_format,
        })
    }

    pub fn host(&self) -> IpAddr {
        self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn number_thread(&self) -> u16 {
        self.number_thread
    }

    pub fn auth_strategy(&self) -> &AuthStrategy {
        &self.auth_strategy
    }

    pub fn log_format(&self) -> &str {
        &self.log_format
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationConfigurations {
    upload_configs: UploadConfiguration,
    ui_configs: UiConfiguration,
    server_configs: ServerConfiguration,
}

impl ApplicationConfigurations {
    pub fn from_config_file(some_conf_file: Option<&String>) -> Result<ApplicationConfigurations, AppError> {
        let configs;

        if let Some(config_file) = some_conf_file {
            configs = Ini::load_from_file(config_file)
                .expect(
                    format!("Could not read config file: {}. You can set environment variable (FSHARE_CONF_FILE) with the filename or pass by argument",
                            config_file).as_str())
        } else {
            let config_file = std::env::var(FSHARE_CONF_FILE_ENV_KEY)
                .ok()
                .map(|d| std::path::Path::new(&d).to_path_buf())
                .unwrap_or(std::path::Path::new("config.ini").to_path_buf());

            configs = Ini::load_from_file(&config_file)
                .expect(
                    format!("Could not read config file: {}. You can set environment variable (FSHARE_CONF_FILE) with the filename or pass by argument",
                            &config_file.display()).as_str());
        }

        Ok(ApplicationConfigurations {
            server_configs: ServerConfiguration::try_from(&configs)?,
            upload_configs: UploadConfiguration::try_from(&configs)?,
            ui_configs: UiConfiguration::try_from(&configs)?,
        })
    }

    pub fn upload_configs(&self) -> &UploadConfiguration {
        &self.upload_configs
    }

    pub fn ui_configs(&self) -> &UiConfiguration {
        &self.ui_configs
    }

    pub fn server_configs(&self) -> &ServerConfiguration {
        &self.server_configs
    }
}
