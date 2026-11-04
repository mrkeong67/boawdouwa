use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "case_type", rename_all = "lowercase")]
pub enum CaseType {
    Warn,
    Strike,
    Timeout,
    Kick,
    Ban,
    Unban,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct ModCase {
    pub id: Uuid,
    pub guild_id: String,
    pub user_id: String,
    pub moderator_id: String,
    pub r#type: CaseType,
    pub reason: String,
    pub evidence_url: Option<String>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct ModSettings {
    pub guild_id: String,
    pub auto_mod_enabled: bool,
    pub anti_spam_enabled: bool,
    pub anti_link_enabled: bool,
    pub anti_raid_enabled: bool,
    pub anti_nuke_enabled: bool,
    pub strike_ban_threshold: i32,
    pub strike_kick_threshold: i32,
    pub log_channel_id: Option<String>,
}

pub struct ModerationRepo {
    pool: PgPool,
}

impl ModerationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_settings(&self, guild_id: &str) -> Result<ModSettings, sqlx::Error> {
        let settings = sqlx::query_as::<_, ModSettings>(
            r#"
            SELECT guild_id, auto_mod_enabled, anti_spam_enabled, anti_link_enabled, 
                   anti_raid_enabled, anti_nuke_enabled, strike_ban_threshold, 
                   strike_kick_threshold, log_channel_id
            FROM mod_settings 
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(s) = settings {
            Ok(s)
        } else {
            let default_settings = sqlx::query_as::<_, ModSettings>(
                r#"
                INSERT INTO mod_settings (guild_id) VALUES ($1)
                RETURNING guild_id, auto_mod_enabled, anti_spam_enabled, anti_link_enabled, 
                          anti_raid_enabled, anti_nuke_enabled, strike_ban_threshold, 
                          strike_kick_threshold, log_channel_id
                "#,
            )
            .bind(guild_id)
            .fetch_one(&self.pool)
            .await?;
            Ok(default_settings)
        }
    }

    pub async fn create_case(&self, case: &ModCase) -> Result<ModCase, sqlx::Error> {
        let _ = sqlx::query(
            "INSERT INTO members (guild_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(&case.guild_id)
        .bind(&case.user_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, ModCase>(
            r#"
            INSERT INTO mod_cases (guild_id, user_id, moderator_id, type, reason, evidence_url, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, guild_id, user_id, moderator_id, type, reason, evidence_url, is_active, expires_at, created_at
            "#,
        )
        .bind(&case.guild_id)
        .bind(&case.user_id)
        .bind(&case.moderator_id)
        .bind(&case.r#type)
        .bind(&case.reason)
        .bind(&case.evidence_url)
        .bind(case.expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_cases(&self, guild_id: &str, user_id: &str) -> Result<Vec<ModCase>, sqlx::Error> {
        sqlx::query_as::<_, ModCase>(
            r#"
            SELECT id, guild_id, user_id, moderator_id, type, reason, evidence_url, is_active, expires_at, created_at
            FROM mod_cases
            WHERE guild_id = $1 AND user_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
}
