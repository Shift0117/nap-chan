use std::{fs::File, io::Write};

use anyhow::{anyhow, Result};
use poise::{async_trait, serenity_prelude::Cache};
use reqwest;
use serenity::{
    client::Context,
    model::id::GuildId,
    utils::{content_safe, ContentSafeOptions},
};
use sqlx::{Pool, Sqlite};
use tempfile;

use super::text::TextMessage;

#[derive(Debug)]
pub struct VoiceOptions<T: VoiceGenerator> {
    generator: T,
    volume: f32,
    speed_auto_scaling: bool,
}

#[derive(Debug)]
pub struct TextOptions<'a, 'b, 'c> {
    clean: Option<&'a ContentSafeOptions>,
    dict: Option<&'b Pool<Sqlite>>,
    read_name: Option<&'c String>,
}

#[async_trait]
pub trait VoiceGenerator {
    async fn create_voice(&self, text: &str, file: &mut File) -> Result<()>;
}

pub struct VoiceVoxAPI {
    url: String,
    voice_type: i64,
}

impl VoiceVoxAPI {
    pub fn new(url: String, voice_type: i64) -> Self {
        Self { url, voice_type }
    }
}

pub struct WebVoiceVoxAPI {
    url: String,
    api_key: String,
    voice_type: i64,
}

impl WebVoiceVoxAPI {
    pub fn new(url: String, api_key: String, voice_type: i64) -> Self {
        Self {
            url,
            api_key,
            voice_type,
        }
    }
}

#[async_trait]
impl VoiceGenerator for VoiceVoxAPI {
    async fn create_voice(&self, text: &str, file: &mut File) -> Result<()> {
        let params = [("text", text), ("speaker", &self.voice_type.to_string())];
        let client = reqwest::Client::new();
        let voice_query_url = format!("{}/audio_query", self.url);
        let res = client.post(voice_query_url).query(&params).send().await?;
        let synthesis_body = res.text().await?;
        let synthesis_arg = [("speaker", self.voice_type)];
        let synthesis_url = format!("{}/synthesis", self.url);
        let synthesis_res = client
            .post(synthesis_url)
            .body(synthesis_body)
            .query(&synthesis_arg)
            .send()
            .await?;
        let _ = file.write(&synthesis_res.bytes().await?)?;
        Ok(())
    }
}

#[async_trait]
impl VoiceGenerator for WebVoiceVoxAPI {
    async fn create_voice(&self, text: &str, file: &mut File) -> Result<()> {
        dotenv::dotenv().ok();
        let params = [
            ("key", self.api_key.as_str()),
            ("text", text),
            ("speaker", &self.voice_type.to_string()),
        ];
        let client = reqwest::Client::new();
        let res = client
            .post(&self.url)
            .query(&params)
            .send()
            .await?
            .bytes()
            .await?;
        let _ = file.write(&res)?;
        Ok(())
    }
}

impl<'a, 'b, 'c> TextOptions<'a, 'b, 'c> {
    pub fn new() -> Self {
        Self {
            clean: None,
            dict: None,
            read_name: None,
        }
    }
    pub fn read_name(&mut self, read_name: Option<&'c String>) -> &mut Self {
        self.read_name = read_name;
        self
    }
    pub fn clean(&mut self, clean: Option<&'a ContentSafeOptions>) -> &mut Self {
        self.clean = clean;
        self
    }
    pub fn dict(&mut self, dict: Option<&'b Pool<Sqlite>>) -> &mut Self {
        self.dict = dict;
        self
    }
    pub async fn format(&self, cache: &std::sync::Arc<Cache>, mut str: String) -> String {
        if let Some(read_name) = self.read_name {
            str = format!("{} {}", read_name, str);
        }
        if let Some(options) = self.clean {
            str = content_safe(cache, str, options, &[]);
        }
        if let Some(dict) = self.dict {
            str = str.make_read_text(dict).await;
        }
        if str.chars().all(|c| !c.is_alphanumeric()) {
            str = "".to_string();
        }
        str
    }
}

impl<T: VoiceGenerator> VoiceOptions<T> {
    pub fn new(voice_generator: T) -> Self {
        Self {
            generator: voice_generator,
            volume: 1.,
            speed_auto_scaling: false,
        }
    }
    pub fn speed_auto_scaling(&mut self, flag: bool) -> &mut Self {
        self.speed_auto_scaling = flag;
        self
    }

    pub async fn play_voice<'a, 'b, 'c>(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        str: String,
    ) -> Result<()> {
        // tracing::info!("voice setting: {:?}", &self);
        if str.is_empty() {
            return Ok(());
        }
        let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
        self.generator
            .create_voice(&str, temp_file.as_file_mut())
            .await?;

        let (_, path) = temp_file.keep()?;
        let manager = songbird::get(ctx)
            .await
            .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?
            .clone();
        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            let mut source = songbird::ffmpeg(&path).await?;
            source.metadata.source_url = Some(path.to_string_lossy().to_string());
            let (mut track, _) = songbird::tracks::create_player(source);
            track.set_volume(self.volume);
            handler.enqueue(track);
        }
        Ok(())
    }
}

// pub async fn play_raw_voice(
//     ctx: &Context,
//     str: &str,
//     voice_type: u32,
//     generator_type: usize,
//     guild_id: GuildId,
// ) -> Result<()> {
//     let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
//     create_voice(str, voice_type, generator_type, temp_file.as_file_mut()).await?;
//     let (_, path) = temp_file.keep()?;
//     let manager = songbird::get(ctx)
//         .await
//         .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?
//         .clone();
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let mut handler = handler_lock.lock().await;
//         let mut source = songbird::ffmpeg(&path).await?;
//         source.metadata.source_url = Some(path.to_string_lossy().to_string());
//         handler.enqueue_source(source);
//     }
//     Ok(())
// }
