use std::sync::OnceLock;

pub static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();
