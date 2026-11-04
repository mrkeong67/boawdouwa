use std::sync::Arc;

pub const ROLE_VETERAN_ID: u64 = 1558424052857708575;
pub const ROLE_MEMBER_ID: u64 = 1457394591019041050;
pub const ROLE_GUEST_ID: u64 = 1556000057969610752;

pub struct Data {
    pub chatbot_enabled: Arc<tokio::sync::RwLock<bool>>,
    pub db_pool: sqlx::PgPool,
    pub chat_history: Arc<tokio::sync::RwLock<std::collections::HashMap<u64, Vec<serde_json::Value>>>>,
    pub clan_data: Arc<tokio::sync::RwLock<String>>,
    pub start_time: std::time::Instant,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
