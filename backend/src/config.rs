use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ReleasePaths {
    pub release_dir: PathBuf,
    pub config_file: PathBuf,
    pub dist_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl ReleasePaths {
    pub fn from_release_dir(release_dir: impl AsRef<Path>) -> Self {
        let release_dir = release_dir.as_ref().to_path_buf();

        Self {
            config_file: release_dir.join("config/.env"),
            dist_dir: release_dir.join("dist"),
            logs_dir: release_dir.join("logs"),
            release_dir,
        }
    }
}

pub fn load_database_url(paths: &ReleasePaths) -> Result<String, Box<dyn std::error::Error>> {
    dotenvy::from_path(&paths.config_file)?;

    dotenvy::from_path_iter(&paths.config_file)?
        .find_map(|entry| match entry {
            Ok((key, value)) if key == "DATABASE_URL" => Some(Ok(value)),
            Ok(_) => None,
            Err(error) => Some(Err(Box::new(error) as Box<dyn std::error::Error>)),
        })
        .unwrap_or_else(|| {
            Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "DATABASE_URL is missing from release configuration",
            )))
        })
}
