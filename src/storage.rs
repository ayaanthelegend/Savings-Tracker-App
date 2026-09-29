use crate::models::AppData;
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct StorageManager {
    file_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SavingsTracker");

        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!("Failed to create storage directory {:?}: {}", dir, e);
        }

        let file_path = dir.join("data.json");
        Self { file_path }
    }

    #[allow(dead_code)]
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Loads app data from disk, or returns default seeded data if not found.
    pub fn load(&self) -> AppData {
        if !self.file_path.exists() {
            let default_data = AppData::default();
            let _ = self.save(&default_data);
            return default_data;
        }

        match fs::read_to_string(&self.file_path) {
            Ok(content) => match serde_json::from_str::<AppData>(&content) {
                Ok(mut data) => {
                    // Ensure there's always at least one card and default categories
                    if data.cards.is_empty() {
                        let def = AppData::default();
                        data.cards = def.cards;
                    }
                    // Ensure default categories exist
                    let defaults = ["Food", "Income", "Subscription", "Transport", "Shopping", "Other"];
                    for cat in defaults {
                        if !data.custom_categories.iter().any(|c| c.eq_ignore_ascii_case(cat)) {
                            data.custom_categories.push(cat.to_string());
                        }
                    }
                    data.ensure_sync_fields();
                    data
                }
                Err(err) => {
                    eprintln!("Failed to parse data JSON: {}. Using default data.", err);
                    AppData::default()
                }
            },
            Err(err) => {
                eprintln!("Failed to read data file: {}. Using default data.", err);
                AppData::default()
            }
        }
    }

    /// Atomically saves data to disk by writing to a temporary file first and renaming it.
    pub fn save(&self, data: &AppData) -> Result<()> {
        let parent = self.file_path.parent().context("No parent directory")?;
        fs::create_dir_all(parent).context("Failed to create parent directory")?;

        let tmp_path = self.file_path.with_extension("tmp");
        let serialized = serde_json::to_string_pretty(data).context("Serialization error")?;

        {
            let mut file = File::create(&tmp_path).context("Failed to create temp file")?;
            file.write_all(serialized.as_bytes())
                .context("Failed to write to temp file")?;
            file.sync_all().context("Failed to sync temp file")?;
        }

        fs::rename(&tmp_path, &self.file_path).context("Failed to rename temp file to target")?;
        Ok(())
    }
}
