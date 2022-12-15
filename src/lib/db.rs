use serde::Deserialize;
use serenity::async_trait;
use sqlx::{query, query_as};
use tracing::info;

use crate::Dict;
use anyhow::{anyhow, Result};
const VOICEVOX:i64 = 0;
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
#[derive(Debug, Clone)]
pub struct VoiceType {
    pub name: String,
    pub style_id: u64,
    pub style_name: String,
    pub generator_type: i64,
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
            Err(_) => {
                query!("INSERT INTO user_config (user_id) VALUES (?)", user_id)
                    .execute(self)
                    .await?;
                Ok(UserConfig::from_user_id(user_id))
            }
            Ok(q) => Ok(q),
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

pub async fn get_voice_types_by_web_voicevox_api() -> Result<Vec<VoiceType>> {
    #[derive(Deserialize, Clone, Debug)]
    struct Style {
        pub name: String,
        pub id: u64,
    }
    #[derive(Deserialize, Clone, Debug)]
    struct Speaker {
        pub name: String,
        pub styles: Vec<Style>,
    }


    let mut voice_types = Vec::new();

   dotenv::dotenv().ok();
        let speakers: Result<Vec<Speaker>> = async {
            let api_key = std::env::var("WEB_API_KEY")?;
            let query_url = "https://api.su-shiki.com/v2/voicevox/speakers/";
            let client = reqwest::Client::new();
            let res = client.get(query_url).query(&[("key",api_key)]).send().await?;
            res.json().await.map_err(|e| e.into())
        }
        .await;
        info!("{:?}", &speakers);
        if let Ok(speakers) = speakers {
            for speaker in speakers {
                for style in speaker.styles {
                    voice_types.push(VoiceType {
                        name: speaker.name.clone(),
                        style_id: style.id,
                        style_name: style.name,
                        generator_type: VOICEVOX,
                    });
                }
            }
        }
    
    Ok(voice_types)
}

pub async fn get_voice_types() -> Result<Vec<VoiceType>> {
    #[derive(Deserialize, Clone, Debug)]
    struct Style {
        pub name: String,
        pub id: u64,
    }
    #[derive(Deserialize, Clone, Debug)]
    struct Speaker {
        pub name: String,
        pub styles: Vec<Style>,
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("generators.json")?;

    let generators: Vec<String> = serde_json::from_reader(file)?;

    info!("generators = {:?}", &generators);
    let mut voice_types = Vec::new();

    for (generator_type, url) in generators.iter().enumerate() {
        let speakers: Result<Vec<Speaker>> = async {
            let query_url = format!("{}/speakers", url);
            let client = reqwest::Client::new();
            let res = client.get(query_url).send().await?;
            res.json().await.map_err(|e| e.into())
        }
        .await;
        info!("{:?}", &speakers);
        if let Ok(speakers) = speakers {
            for speaker in speakers {
                for style in speaker.styles {
                    voice_types.push(VoiceType {
                        name: speaker.name.clone(),
                        style_id: style.id,
                        style_name: style.name,
                        generator_type: generator_type as i64,
                    });
                }
            }
        }
    }
    Ok(voice_types)
}
