use std::{
    fs::{self, File},
    io::Write,
};

use serde::Deserialize;
use serenity::async_trait;
use sqlx::{query, query_as, Acquire};
use tracing::info;

use crate::{handler::Generators, Dict};
use anyhow::{anyhow, Result};

#[async_trait]
pub trait UserConfigDB {
    async fn get_user_config_or_default(&self, user_id: i64) -> UserConfig;
    async fn get_user_config(&self, user_id: i64) -> Result<UserConfig>;
    async fn update_user_config(&self, user_config: &UserConfig) -> u64;
}

#[derive(Debug)]
pub struct UserConfig {
    pub user_id: i64,
    pub hello: String,
    pub bye: String,
    pub voice_type: i64,
    pub generator_type: i64,
    pub read_nickname: Option<String>,
}

pub struct VoiceType {
    pub id: i64,
    pub name: String,
    pub style_id: i64,
    pub style_name: String,
    pub generator_type: String,
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
        query!("UPDATE user_config SET hello = ?,bye = ?,voice_type = ?,generator_type = ?,read_nickname = ? WHERE user_id = ?",
        user_config.hello,user_config.bye,user_config.voice_type,user_config.generator_type,user_config.read_nickname,user_config.user_id)
        .execute(self).await.map_or(0, |result| result.rows_affected())
    }
}

#[async_trait]
pub trait DictDB {
    async fn update_dict(&self, dict: &Dict) -> u64;
    async fn get_dict(&self, word: &str) -> Result<String>;
    async fn get_dict_all(&self) -> Vec<Dict>;
    async fn remove(&self, word: &str) -> Result<()>;
}

#[async_trait]
impl DictDB for sqlx::SqlitePool {
    async fn update_dict(&self, dict: &Dict) -> u64 {
        let mut tx = self.begin().await.unwrap();
        query!(
            "INSERT OR REPLACE INTO dict VALUES (?,?)",
            dict.word,
            dict.read_word
        )
        .execute(&mut tx)
        .await
        .map_or(0, |result| result.rows_affected())
    }
    async fn get_dict(&self, word: &str) -> Result<String> {
        let mut tx = self.begin().await.unwrap();
        Ok(query!("SELECT read_word FROM dict WHERE word = ?", word)
            .fetch_optional(&mut tx)
            .await?
            .ok_or(anyhow!("key not found"))?
            .read_word)
    }
    async fn get_dict_all(&self) -> Vec<Dict> {
        let mut tx = self.begin().await.unwrap();
        sqlx::query_as!(Dict, "SELECT word,read_word FROM dict")
            .fetch_all(&mut tx)
            .await
            .unwrap()
    }
    async fn remove(&self, word: &str) -> Result<()> {
        let mut tx = self.begin().await.unwrap();
        sqlx::query!("DELETE FROM dict WHERE word = ?", word)
            .execute(&mut tx)
            .await
            .map_err(|e| e.into())
            .map(|_| ())
        //tx.commit().await
    }
}

#[async_trait]
pub trait SpeakerDB {
    async fn speaker_name_to_id(&self, name: &str) -> Result<(Generators, u8)>;
    async fn speaker_id_to_name(&self, generator_type: Generators, id: u8) -> Result<String>;
    async fn generate_speaker_db(&self) -> Result<()>;
    async fn get_speaker(&self,id:usize) -> Result<VoiceType>;
}

#[async_trait]
impl SpeakerDB for sqlx::SqlitePool {
    async fn speaker_name_to_id(&self, name: &str) -> Result<(Generators, u8)> {
        let mut tx = self.begin().await.unwrap();
        let q = query!(
            "SELECT generator_type,style_id FROM speakers WHERE style_name = ?",
            name
        )
        .fetch_one(&mut tx)
        .await?;
        Ok((
            Generators::try_from(q.generator_type.as_str())?,
            q.style_id as u8,
        ))
    }
    async fn speaker_id_to_name(&self, generator_type: Generators, id: u8) -> Result<String> {
        let mut tx = self.begin().await.unwrap();
        let str: &str = generator_type.into();
        let q = query!(
            "SELECT name,style_name FROM speakers WHERE generator_type = ? AND style_id = ?",
            str,
            id
        )
        .fetch_one(&mut tx)
        .await?;
        Ok(format!("{} {}", q.name, q.style_name))
    }
    async fn generate_speaker_db(&self) -> Result<()> {
        dotenv::dotenv().ok();
        #[derive(Deserialize, Clone)]
        struct Style {
            pub name: String,
            pub id: u8,
        }
        #[derive(Deserialize, Clone)]
        struct Speaker {
            pub name: String,
            pub styles: Vec<Style>,
        }

        let mut tx = self.begin().await.unwrap();
        let voicevox_voice_types: Vec<Speaker> = {
            let base_url =
                std::env::var("BASE_URL_VOICEVOX").expect("environment variable not found");
            let query_url = format!("{}/speakers", base_url);
            let client = reqwest::Client::new();
            let res = client
                .get(query_url)
                .send()
                .await
                .expect("Panic in speaker info query");

            //info!("{:?}",res.text().await);
            res.json().await.unwrap()
        };
        for speaker in voicevox_voice_types {
            for style in speaker.styles {
                if let None = query!(
                    "SELECT * FROM speakers WHERE generator_type = ? AND style_id = ?",
                    "VOICEVOX",
                    style.id
                )
                .fetch_optional(&mut tx)
                .await?
                {
                    query!(
                        "INSERT INTO speakers (name,style_id,style_name,generator_type) VALUES (?,?,?,?)",
                        speaker.name,
                        style.id,
                        style.name,
                        "VOICEVOX"
                    )
                    .execute(&mut tx)
                    .await
                    .unwrap();
                }
            }
        }
        let coeiro_voice_types: Vec<Speaker> = {
            let base_url =
                std::env::var("BASE_URL_COEIRO").expect("environment variable not found");
            let query_url = format!("{}/speakers", base_url);
            let client = reqwest::Client::new();
            let res = client
                .get(query_url)
                .send()
                .await
                .expect("Panic in speaker info query");
            res.json().await.unwrap()
        };
        for speaker in coeiro_voice_types {
            for style in speaker.styles {
                if let None = query!(
                    "SELECT * FROM speakers WHERE generator_type = ? AND style_id = ?",
                    "COEIROINK",
                    style.id
                )
                .fetch_optional(&mut tx)
                .await?
                {
                    query!(
                        "INSERT INTO speakers (name,style_id,style_name,generator_type) VALUES (?,?,?,?)",
                        speaker.name,
                        style.id,
                        style.name,
                        "COEIROINK"
                    )
                    .execute(&mut tx)
                    .await
                    .unwrap();
                }
            }
        }
        Ok(())
    }
    async fn get_speaker(&self,id:usize) -> Result<VoiceType> {
        let id = id as i64;
        query_as!(VoiceType,"SELECT * FROM speakers WHERE id = ?",id).fetch_one(self).await.map_err(|e| e.into())
    }
}
