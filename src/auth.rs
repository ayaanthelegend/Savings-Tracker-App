use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        // Check environment variables first, falling back to dedicated project defaults
        let url = std::env::var("SUPABASE_URL")
            .unwrap_or_else(|_| "https://rclbsnojntbhjkphsrhv.supabase.co".to_string());
        let anon_key = std::env::var("SUPABASE_ANON_KEY")
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InJjbGJzbm9qbnRiaGprcGhzcmh2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3OTA2ODY3MDYsImV4cCI6MjEwNjI2MjcwNn0.-H_-lbQ6AVwk8XaZlAq64Xmf1vIPbnwiUueCq8DGGRQ".to_string());
        Self { url, anon_key }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64, // Unix timestamp in seconds
    pub user_id: Uuid,
    pub user_email: String,
}

impl AuthSession {
    /// Returns true if the token is already expired or has less than 5 minutes (300s) of validity left.
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= (self.expires_at - 300)
    }

    /// Returns true if the token is completely expired.
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.expires_at
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials(String),
    TokenRevoked(String),
    Network(String),
    Server(String),
    ConfigMissing(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            AuthError::TokenRevoked(msg) => write!(f, "Session expired/revoked: {}", msg),
            AuthError::Network(msg) => write!(f, "Network error: {}", msg),
            AuthError::Server(msg) => write!(f, "Server error: {}", msg),
            AuthError::ConfigMissing(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

// Supabase response types
#[derive(Deserialize)]
struct SupabaseAuthResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    expires_at: Option<i64>,
    user: Option<SupabaseUser>,
    error_description: Option<String>,
    msg: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct SupabaseUser {
    id: Uuid,
    email: Option<String>,
}

pub struct AuthManager;

impl AuthManager {
    pub fn get_app_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SavingsTracker")
    }

    pub fn session_file_path() -> PathBuf {
        Self::get_app_dir().join("session.json")
    }

    pub fn config_file_path() -> PathBuf {
        Self::get_app_dir().join("config.json")
    }

    pub fn load_config() -> SupabaseConfig {
        let path = Self::config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<SupabaseConfig>(&content) {
                    if !config.url.trim().is_empty() {
                        return config;
                    }
                }
            }
        }
        SupabaseConfig::default()
    }

    pub fn save_config(config: &SupabaseConfig) -> Result<()> {
        let dir = Self::get_app_dir();
        fs::create_dir_all(&dir).context("Failed to create app data directory")?;
        let path = Self::config_file_path();
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_session() -> Option<AuthSession> {
        let path = Self::session_file_path();
        if !path.exists() {
            return None;
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<AuthSession>(&content) {
                Ok(session) => Some(session),
                Err(e) => {
                    eprintln!("Failed to parse session JSON: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to read session file: {}", e);
                None
            }
        }
    }

    pub fn save_session(session: &AuthSession) -> Result<()> {
        let dir = Self::get_app_dir();
        fs::create_dir_all(&dir).context("Failed to create app data directory")?;
        let path = Self::session_file_path();
        let tmp_path = path.with_extension("tmp");
        let content = serde_json::to_string_pretty(session)?;

        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(tmp_path, path)?;
        Ok(())
    }

    pub fn clear_session() -> Result<()> {
        let path = Self::session_file_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }

    fn clean_url(url: &str) -> String {
        let mut u = url.trim().to_string();
        while u.ends_with('/') {
            u.pop();
        }
        u
    }

    /// Authenticates with Supabase using email and password.
    pub fn login(
        client: &reqwest::blocking::Client,
        config: &SupabaseConfig,
        email: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        let base_url = Self::clean_url(&config.url);
        if base_url.is_empty() || config.anon_key.trim().is_empty() {
            return Err(AuthError::ConfigMissing(
                "Supabase URL or Anon Key is missing. Please configure them in Settings or on the login page.".to_string(),
            ));
        }

        let endpoint = format!("{}/auth/v1/token?grant_type=password", base_url);
        let payload = serde_json::json!({
            "email": email.trim(),
            "password": password,
        });

        let resp = client
            .post(&endpoint)
            .header("apikey", config.anon_key.trim())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        let status = resp.status();
        let body_text = resp.text().map_err(|e| AuthError::Network(e.to_string()))?;

        if status.is_success() {
            let parsed: SupabaseAuthResponse = serde_json::from_str(&body_text)
                .map_err(|e| AuthError::Server(format!("Invalid auth response JSON: {}", e)))?;

            let access_token = parsed
                .access_token
                .ok_or_else(|| AuthError::Server("Missing access_token in response".into()))?;
            let refresh_token = parsed
                .refresh_token
                .ok_or_else(|| AuthError::Server("Missing refresh_token in response".into()))?;
            let user = parsed
                .user
                .ok_or_else(|| AuthError::Server("Missing user object in response".into()))?;

            let now = Utc::now().timestamp();
            let expires_at = parsed.expires_at.unwrap_or_else(|| {
                now + parsed.expires_in.unwrap_or(3600)
            });

            let session = AuthSession {
                access_token,
                refresh_token,
                expires_at,
                user_id: user.id,
                user_email: user.email.unwrap_or_else(|| email.trim().to_string()),
            };

            let _ = Self::save_session(&session);
            Ok(session)
        } else {
            let error_msg = if let Ok(err_obj) = serde_json::from_str::<SupabaseAuthResponse>(&body_text) {
                err_obj
                    .error_description
                    .or(err_obj.msg)
                    .or(err_obj.message)
                    .or(err_obj.error)
                    .unwrap_or_else(|| format!("HTTP status {}", status))
            } else {
                format!("HTTP error {}: {}", status, body_text)
            };

            if status.as_u16() == 400 || status.as_u16() == 401 {
                Err(AuthError::InvalidCredentials(error_msg))
            } else {
                Err(AuthError::Server(error_msg))
            }
        }
    }

    /// Creates a new user account with Supabase using email and password.
    pub fn signup(
        client: &reqwest::blocking::Client,
        config: &SupabaseConfig,
        email: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        let base_url = Self::clean_url(&config.url);
        if base_url.is_empty() || config.anon_key.trim().is_empty() {
            return Err(AuthError::ConfigMissing(
                "Supabase URL or Anon Key is missing.".to_string(),
            ));
        }

        let endpoint = format!("{}/auth/v1/signup", base_url);
        let payload = serde_json::json!({
            "email": email.trim(),
            "password": password,
        });

        let resp = client
            .post(&endpoint)
            .header("apikey", config.anon_key.trim())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        let status = resp.status();
        let body_text = resp.text().map_err(|e| AuthError::Network(e.to_string()))?;

        if status.is_success() {
            let parsed: SupabaseAuthResponse = serde_json::from_str(&body_text)
                .map_err(|e| AuthError::Server(format!("Invalid auth response: {}", e)))?;

            if let (Some(access_token), Some(refresh_token), Some(user)) =
                (parsed.access_token, parsed.refresh_token, parsed.user)
            {
                let now = Utc::now().timestamp();
                let expires_at = parsed.expires_at.unwrap_or_else(|| {
                    now + parsed.expires_in.unwrap_or(3600)
                });

                let session = AuthSession {
                    access_token,
                    refresh_token,
                    expires_at,
                    user_id: user.id,
                    user_email: user.email.unwrap_or_else(|| email.trim().to_string()),
                };

                let _ = Self::save_session(&session);
                Ok(session)
            } else {
                // Email confirmation is required by Supabase project settings
                Err(AuthError::Server(
                    "Sign-up successful! Please check your email to confirm your account before logging in, or disable 'Confirm email' in Supabase Auth settings."
                        .to_string(),
                ))
            }
        } else {
            let error_msg = if let Ok(err_obj) = serde_json::from_str::<SupabaseAuthResponse>(&body_text) {
                err_obj
                    .error_description
                    .or(err_obj.msg)
                    .or(err_obj.message)
                    .or(err_obj.error)
                    .unwrap_or_else(|| format!("HTTP status {}", status))
            } else {
                format!("HTTP error {}: {}", status, body_text)
            };

            Err(AuthError::Server(error_msg))
        }
    }

    /// Refreshes the session using the stored refresh token.
    pub fn refresh_session(
        client: &reqwest::blocking::Client,
        config: &SupabaseConfig,
        refresh_token: &str,
    ) -> Result<AuthSession, AuthError> {
        let base_url = Self::clean_url(&config.url);
        if base_url.is_empty() || config.anon_key.trim().is_empty() {
            return Err(AuthError::ConfigMissing("Supabase config missing".to_string()));
        }

        let endpoint = format!("{}/auth/v1/token?grant_type=refresh_token", base_url);
        let payload = serde_json::json!({
            "refresh_token": refresh_token,
        });

        let resp = client
            .post(&endpoint)
            .header("apikey", config.anon_key.trim())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        let status = resp.status();
        let body_text = resp.text().map_err(|e| AuthError::Network(e.to_string()))?;

        if status.is_success() {
            let parsed: SupabaseAuthResponse = serde_json::from_str(&body_text)
                .map_err(|e| AuthError::Server(format!("Invalid refresh response: {}", e)))?;

            let access_token = parsed
                .access_token
                .ok_or_else(|| AuthError::Server("Missing access_token in response".into()))?;
            let new_refresh_token = parsed
                .refresh_token
                .unwrap_or_else(|| refresh_token.to_string());
            let user = parsed
                .user
                .ok_or_else(|| AuthError::Server("Missing user object in response".into()))?;

            let now = Utc::now().timestamp();
            let expires_at = parsed.expires_at.unwrap_or_else(|| {
                now + parsed.expires_in.unwrap_or(3600)
            });

            let session = AuthSession {
                access_token,
                refresh_token: new_refresh_token,
                expires_at,
                user_id: user.id,
                user_email: user.email.unwrap_or_default(),
            };

            let _ = Self::save_session(&session);
            Ok(session)
        } else {
            let error_msg = if let Ok(err_obj) = serde_json::from_str::<SupabaseAuthResponse>(&body_text) {
                err_obj
                    .error_description
                    .or(err_obj.msg)
                    .or(err_obj.message)
                    .or(err_obj.error)
                    .unwrap_or_else(|| format!("HTTP status {}", status))
            } else {
                format!("HTTP error {}: {}", status, body_text)
            };

            if status.as_u16() == 400 || status.as_u16() == 401 {
                Err(AuthError::TokenRevoked(error_msg))
            } else {
                Err(AuthError::Server(error_msg))
            }
        }
    }
}
