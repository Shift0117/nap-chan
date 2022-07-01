use serenity::async_trait;
use sqlx::{query, query_as};

use crate::{Dict, UserConfig};
use anyhow::{anyhow, Result};

#[async_trait]
pub trait UserConfigDB {
    async fn get_user_config_or_default(&self, user_id: i64) -> UserConfig;
    async fn get_user_config(&self, user_id: i64) -> Result<UserConfig>;
    async fn update_user_config(&self, user_config: &UserConfig) -> u64;
}
#[async_trait]
impl UserConfigDB for sqlx::SqlitePool {
    async fn get_user_config(&self, user_id: i64) -> Result<UserConfig> {
        query_as!(
            UserConfig,
            "SELECT * FROM user_config WHERE user_id = ?",
            user_id
        )
        .fetch_optional(self)
        .await?
        .ok_or(anyhow!("key not found"))
    }
    async fn get_user_config_or_default(&self, user_id: i64) -> UserConfig {
        match self.get_user_config(user_id).await {
            Ok(q) => q,
            Err(_) => {
                query!("INSERT INTO user_config (user_id) VALUES (?)", user_id)
                    .execute(self)
                    .await
                    .ok();
                self.get_user_config(user_id).await.unwrap()
            }
        }
    }
    async fn update_user_config(&self, user_config: &UserConfig) -> u64 {
        query!("UPDATE user_config SET hello = ?,bye = ?,voice_type = ?,generator_type = ? WHERE user_id = ?",
        user_config.hello,user_config.bye,user_config.voice_type,user_config.generator_type,user_config.user_id)
        .execute(self).await.map_or(0, |result| result.rows_affected())
    }
}

#[async_trait]
pub trait DictDB {
    async fn update_dict(&self, dict: &Dict) -> u64;
    async fn get_dict(&self, word: &str) -> Result<String>;
    async fn get_dict_all(&self) -> Vec<Dict>;
}

#[async_trait]
impl DictDB for sqlx::SqlitePool {
    async fn update_dict(&self, dict: &Dict) -> u64 {
        query!(
            "INSERT OR REPLACE INTO dict VALUES (?,?)",
            dict.word,
            dict.read_word
        )
        .execute(self)
        .await
        .map_or(0, |result| result.rows_affected())
    }
    async fn get_dict(&self, word: &str) -> Result<String> {
        Ok(query!("SELECT read_word FROM dict WHERE word = ?", word)
            .fetch_optional(self)
            .await?
            .ok_or(anyhow!("key not found"))?
            .read_word)
    }
    async fn get_dict_all(&self) -> Vec<Dict> {
        sqlx::query_as!(Dict, "SELECT word,read_word FROM dict")
            .fetch_all(self)
            .await
            .unwrap()
    }
}
