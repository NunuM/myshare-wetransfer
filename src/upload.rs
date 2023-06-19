use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use actix_multipart::Multipart;
use actix_web::web;
use actix_web::web::Buf;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;

use crate::errors::AppError;
use crate::utils::generate_random_link;
use actix_files::NamedFile;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub enum FileType {
    Regular,
    Archive(String),
}

#[derive(Serialize, Clone)]
pub struct FileInfo {
    name: String,
    file_type: FileType,
    size: u64,
    created: u64,
}

impl FileInfo {
    fn new(name: String, file_type: FileType, size: u64, created: u64) -> Self {
        FileInfo {
            name,
            file_type,
            size,
            created,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UploadManager {
    destination: PathBuf,
    max_size: usize,
}

impl UploadManager {
    pub fn new(dst: PathBuf, max_size: usize) -> Self {
        UploadManager { destination: dst, max_size }
    }

    pub fn get_file_from_link<F: AsRef<str>>(&self, link: F) -> Result<NamedFile, AppError> {
        Ok(NamedFile::open(format!(
            "{}/{}.zip",
            self.destination.to_string_lossy(),
            link.as_ref()
        ))?)
    }

    pub async fn store(&self, mut payload: Multipart) -> Result<String, AppError> {
        let archive_name = generate_random_link();
        let dest_path = self.destination.clone();

        let mut uploaded: usize = 0;

        let max_size = self.max_size;

        let filename_0 = format!("{}/{}.zip", dest_path.to_string_lossy(), archive_name);
        let filename_1 = filename_0.clone();
        let filename_2 = filename_0.clone();

        let mut is_empty = true;

        let target = web::block(move || std::fs::File::create(filename_0)).await?;

        let mut zipper = zip::ZipWriter::new(target);

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        while let Ok(Some(mut field)) = payload.try_next().await {
            let some_name = field
                .content_disposition()
                .map(|d| d.get_filename().map(|s| s.to_string()))
                .flatten();

            let filename = some_name.unwrap_or(generate_random_link());

            zipper.start_file(filename, options)?;

            while let Some(chunk) = field.next().await {
                let data = chunk?;

                uploaded += data.len();

                if uploaded > max_size {
                    if let Err(err) = web::block(move || std::fs::remove_file(filename_1)).await {
                        error!("Cannot delete file: {}: {:?}", filename_2, err);
                    }

                    return Err(AppError::ArchiveError(format!("File to big")));
                }

                zipper = web::block(move || zipper.write_all(data.bytes()).map(|_| zipper)).await?;

                is_empty = false;
            }
        }

        if is_empty {
            if let Err(err) = web::block(move || std::fs::remove_file(filename_1)).await {
                error!("Cannot delete file: {}: {:?}", filename_2, err);
            }

            return Err(AppError::ArchiveError(format!("Empty file")));
        }

        zipper.finish()?;

        Ok(archive_name)
    }

    pub fn list_directory(&self) -> Result<Vec<FileInfo>, AppError> {
        let mut dirs = Vec::new();
        let dir_entries = std::fs::read_dir(&self.destination)?;

        for entry_result in dir_entries {
            let entry = entry_result?;

            let name = entry.file_name().to_string_lossy().to_string();
            let size = entry.metadata()?.len();
            let created = entry
                .metadata()?
                .created()?
                .duration_since(SystemTime::UNIX_EPOCH)?;

            if name.ends_with("zip") {
                let result_archive = zip::ZipArchive::new(std::fs::File::open(entry.path())?);

                if let Ok(archive) = result_archive {
                    for file in archive.file_names() {
                        dirs.push(FileInfo::new(
                            file.to_string(),
                            FileType::Archive(name.replace(".zip", "").to_string()),
                            size,
                            created.as_secs(),
                        ))
                    }
                }
            } else {
                dirs.push(FileInfo::new(
                    name.to_string(),
                    FileType::Regular,
                    size,
                    created.as_secs(),
                ))
            }
        }

        Ok(dirs)
    }
}

#[derive(Serialize, Clone)]
pub struct DisplayDirectories {
    date: String,
    files: HashMap<String, Vec<FileInfo>>,
}

impl DisplayDirectories {
    pub fn from(data: &Vec<FileInfo>) -> Vec<Self> {
        let mut result = Vec::new();

        let data = data
            .iter()
            .map(|f| {
                let date = chrono::NaiveDateTime::from_timestamp(f.created as i64, 0);

                (date.format("%Y, %m %d").to_string(), f)
            })
            .fold(HashMap::new(), |mut acc, (d, f)| {
                let v = acc.entry(d).or_insert(Vec::new());

                v.push(f);

                acc
            });

        for (date, entries) in data {
            let mut d = DisplayDirectories {
                date,
                files: HashMap::new(),
            };

            for entry in entries {
                let key = match entry.file_type {
                    FileType::Archive(ref name) => name,
                    FileType::Regular => &entry.name,
                };

                let fs = d.files.entry(key.to_string()).or_insert(Vec::new());

                fs.push(entry.clone());
            }

            result.push(d);
        }

        result.sort_by(|a, b| b.date.cmp(&a.date));

        for dir in result.iter_mut() {
            for values in dir.files.values_mut() {
                values.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        result
    }
}
