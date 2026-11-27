use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::env;

use crate::common::error::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_pass: String,
    pub db_name: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub env: String,
    pub enable_swagger: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = env::var("DB_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(5432);
        let db_user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let db_pass = env::var("DB_PASSWORD")
            .map(|p| p.trim_matches('\'').trim_matches('"').to_string())
            .unwrap_or_else(|_| "postgres".to_string());
        let db_name = env::var("DB_NAME").unwrap_or_else(|_| "postgres".to_string());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let encoded_user = db_user.replace('@', "%40").replace(':', "%3A");
            let encoded_pass = db_pass
                .replace('%', "%25")
                .replace('#', "%23")
                .replace('@', "%40")
                .replace(':', "%3A")
                .replace('/', "%2F");
            format!("postgres://{encoded_user}:{encoded_pass}@{host}:{db_port}/{db_name}")
        });

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_key_change_in_production".to_string());

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|h| h.parse::<i64>().ok())
            .unwrap_or(24);

        let env = env::var("ENV").unwrap_or_else(|_| "development".to_string());

        let enable_swagger = env::var("ENABLE_SWAGGER")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Self {
            port,
            host,
            db_port,
            db_user,
            db_pass,
            db_name,
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            env,
            enable_swagger,
        }
    }

    pub fn pg_connect_options(&self) -> Result<PgConnectOptions, AppError> {
        if let Ok(raw_url) = env::var("DATABASE_URL") {
            raw_url
                .parse::<PgConnectOptions>()
                .map_err(|e| AppError::Database(format!("Invalid DATABASE_URL: {e}")))
        } else {
            Ok(PgConnectOptions::new()
                .host(&self.host)
                .port(self.db_port)
                .username(&self.db_user)
                .password(&self.db_pass)
                .database(&self.db_name)
                .ssl_mode(PgSslMode::Prefer))
        }
    }
}
