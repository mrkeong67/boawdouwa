use crate::db::moderation::{ModerationRepo, ModCase, CaseType};
use uuid::Uuid;
use chrono::{Utc, Duration};
use sqlx::PgPool;

pub struct ModerationService {
    repo: ModerationRepo,
}

impl ModerationService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: ModerationRepo::new(pool),
        }
    }

    pub async fn warn_user(&self, guild_id: &str, user_id: &str, mod_id: &str, reason: &str, evidence: Option<String>) -> Result<ModCase, sqlx::Error> {
        let case = ModCase {
            id: Uuid::new_v4(), 
            guild_id: guild_id.to_string(),
            user_id: user_id.to_string(),
            moderator_id: mod_id.to_string(),
            r#type: CaseType::Warn,
            reason: reason.to_string(),
            evidence_url: evidence,
            is_active: true,
            expires_at: Some(Utc::now() + Duration::days(30)),
            created_at: None,
        };
        
        self.repo.create_case(&case).await
    }

    pub async fn strike_user(&self, guild_id: &str, user_id: &str, mod_id: &str, reason: &str, evidence: Option<String>) -> Result<(ModCase, i32), sqlx::Error> {
        let case = ModCase {
            id: Uuid::new_v4(),
            guild_id: guild_id.to_string(),
            user_id: user_id.to_string(),
            moderator_id: mod_id.to_string(),
            r#type: CaseType::Strike,
            reason: reason.to_string(),
            evidence_url: evidence,
            is_active: true,
            expires_at: Some(Utc::now() + Duration::days(90)),
            created_at: None,
        };
        
        let new_case = self.repo.create_case(&case).await?;

        let cases = self.repo.get_user_cases(guild_id, user_id).await?;
        let active_strikes = cases.into_iter().filter(|c| c.r#type == CaseType::Strike && c.is_active).count() as i32;

        Ok((new_case, active_strikes))
    }

    pub async fn get_settings(&self, guild_id: &str) -> Result<crate::db::moderation::ModSettings, sqlx::Error> {
        self.repo.get_settings(guild_id).await
    }
}
