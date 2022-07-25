use serde::Deserialize;
use serenity::async_trait;
use sqlx::{query, query_as};

use crate::{handler::Generators, Dict};
use anyhow::{anyhow, Result};

#[async_trait]
pub trait UserConfigDB {
    async fn get_user_config_or_default(&self, user_id: i64) -> Result<UserConfig>;
    async fn get_user_config(&self, user_id: i64) -> Result<UserConfig>;
    async fn update_user_config(&self, user_config: &UserConfig) -> Result<u64>;
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
impl UserConfig {
    pub fn from_user_id(user_id: i64) -> Self {
        UserConfig {
            user_id,
            hello: "こんにちは".to_string(),
            bye: "ばいばい".to_string(),
            voice_type: 1,
            generator_type: 0,
            read_nickname: None,
        }
    }
}
#[derive(Debug)]
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
        let mut tx = self.begin().await?;
        let q = query_as!(
            UserConfig,
            "SELECT * FROM user_config WHERE user_id = ?",
            user_id
        )
        .fetch_optional(&mut tx)
        .await?
        .ok_or_else(|| anyhow!("key not found"))?;
        tx.commit().await?;
        Ok(q)
    }
    async fn get_user_config_or_default(&self, user_id: i64) -> Result<UserConfig> {
        match self.get_user_config(user_id).await {
            Ok(q) => Ok(q),
            Err(_) => {
                query!("INSERT INTO user_config (user_id) VALUES (?)", user_id)
                    .execute(self)
                    .await?;
                Ok(UserConfig::from_user_id(user_id))
            }
        }
    }
    async fn update_user_config(&self, user_config: &UserConfig) -> Result<u64> {
        let mut tx = self.begin().await?;
        let q = query!("UPDATE user_config SET hello = ?,bye = ?,voice_type = ?,generator_type = ?,read_nickname = ? WHERE user_id = ?",
        user_config.hello,user_config.bye,user_config.voice_type,user_config.generator_type,user_config.read_nickname,user_config.user_id)
        .execute(&mut tx).await?;
        tx.commit().await?;
        Ok(q.rows_affected())
    }
}

#[async_trait]
pub trait DictDB {
    async fn update_dict(&self, dict: &Dict) -> Result<u64>;
    async fn get_dict(&self, word: &str) -> Result<String>;
    async fn get_dict_all(&self) -> Result<Vec<Dict>>;
    async fn remove(&self, word: &str) -> Result<()>;
}

#[async_trait]
impl DictDB for sqlx::SqlitePool {
    async fn update_dict(&self, dict: &Dict) -> Result<u64> {
        let mut tx = self.begin().await?;
        let q = query!(
            "INSERT OR REPLACE INTO dict VALUES (?,?)",
            dict.word,
            dict.read_word
        )
        .execute(&mut tx)
        .await
        .map_or(0, |result| result.rows_affected());
        tx.commit().await?;
        Ok(q)
    }
    async fn get_dict(&self, word: &str) -> Result<String> {
        let mut tx = self.begin().await?;
        let dict = query!("SELECT read_word FROM dict WHERE word = ?", word)
            .fetch_optional(&mut tx)
            .await?
            .ok_or_else(|| anyhow!("key not found"))?
            .read_word;
        tx.commit().await?;
        Ok(dict)
    }
    async fn get_dict_all(&self) -> Result<Vec<Dict>> {
        let mut tx = self.begin().await?;
        let dict = sqlx::query_as!(Dict, "SELECT word,read_word FROM dict")
            .fetch_all(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(dict)
    }
    async fn remove(&self, word: &str) -> Result<()> {
        let mut tx = self.begin().await.unwrap();
        sqlx::query!("DELETE FROM dict WHERE word = ?", word)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
pub trait SpeakerDB {
    async fn speaker_name_to_id(&self, name: &str) -> Result<(Generators, u8)>;
    async fn speaker_id_to_name(&self, generator_type: Generators, id: u8) -> Result<String>;
    async fn insert_speaker_data(&self) -> Result<()>;
    async fn get_speaker(&self, id: usize) -> Result<VoiceType>;
    async fn get_all_speakers(&self) -> Result<Vec<VoiceType>>;
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
        tx.commit().await?;
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
        tx.commit().await?;
        Ok(format!("{} {}", q.name, q.style_name))
    }
    async fn insert_speaker_data(&self) -> Result<()> {
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
        query!("DELETE FROM speakers")
            .execute(&mut tx)
            .await
            .unwrap();
        query!("DELETE FROM sqlite_sequence WHERE name = 'speakers'")
            .execute(&mut tx)
            .await
            .unwrap();
        let voicevox_voice_types: Result<Vec<Speaker>> = async {
            let base_url = std::env::var("BASE_URL_VOICEVOX")?;
            let query_url = format!("{}/speakers", base_url);
            let client = reqwest::Client::new();
            let res = client.get(query_url).send().await?;

            //info!("{:?}",res.text().await);
            res.json().await.map_err(|e| e.into())
        }
        .await;
        if let Ok(voicevox_voice_types) = voicevox_voice_types {
            for speaker in voicevox_voice_types {
                for style in speaker.styles {
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
        let coeiro_voice_types: Result<Vec<Speaker>> = async {
            let base_url = std::env::var("BASE_URL_COEIRO")?;
            let query_url = format!("{}/speakers", base_url);
            let client = reqwest::Client::new();
            let res = client.get(query_url).send().await?;
            res.json().await.map_err(|e| e.into())
        }
        .await;
        if let Ok(coeiro_voice_types) = coeiro_voice_types {
            for speaker in coeiro_voice_types {
                for style in speaker.styles {
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
        tx.commit().await?;
        Ok(())
    }
    async fn get_speaker(&self, id: usize) -> Result<VoiceType> {
        let id = id as i64;
        let mut tx = self.begin().await?;
        let q = query_as!(VoiceType, "SELECT * FROM speakers WHERE id = ?", id)
            .fetch_one(&mut tx)
            .await?;
        tx.commit().await?;

        Ok(q)
    }
    async fn get_all_speakers(&self) -> Result<Vec<VoiceType>> {
        let mut tx = self.begin().await?;
        let q = query_as!(VoiceType, "SELECT * FROM speakers")
            .fetch_all(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(q)
    }
}
