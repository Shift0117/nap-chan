use std::{fs::File, io::Write};

use anyhow::{anyhow, Result};
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
pub struct VoiceOptions<'a, 'b, 'c> {
    clean: Option<&'a ContentSafeOptions>,
    dict: Option<&'b Pool<Sqlite>>,
    read_name: Option<&'c String>,
    voice_type: i64,
    generator_type: i64,
    volume: f32,
    speed_auto_scaling:bool
}

impl<'a, 'b, 'c> VoiceOptions<'a, 'b, 'c> {
    pub fn new() -> Self {
        Self {
            clean: None,
            dict: None,
            read_name: None,
            voice_type: 0,
            generator_type: 0,
            volume: 1.,
            speed_auto_scaling:false
        }
    }
    pub fn speed_auto_scaling(&mut self,flag:bool) -> &mut Self {
        self.speed_auto_scaling = flag;
        self
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
    pub fn voice_type(&mut self, voice_type: i64) -> &mut Self {
        self.voice_type = voice_type;
        self
    }
    pub fn generator_type(&mut self, generator_type: i64) -> &mut Self {
        self.generator_type = generator_type;
        self
    }

    pub async fn play_voice(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        mut str: String,
    ) -> Result<()> {
        tracing::info!("voice setting: {:?}", &self);
        if let Some(read_name) = self.read_name {
            str = format!("{} {}", read_name, str);
        }
        if let Some(options) = self.clean {
            str = content_safe(&ctx.cache, str, options, &[]);
        }
        if let Some(dict) = self.dict {
            str = str.make_read_text(dict).await;
        }
        if str.chars().all(|c| !c.is_alphanumeric()) {
            return Ok(());
        }
        let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
        create_voice(
            &str,
            self.voice_type as u32,
            self.generator_type as usize,
            temp_file.as_file_mut(),
        )
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

pub async fn create_voice(
    text: &str,
    voice_type: u32,
    generator_type: usize,
    temp_file: &mut File,
) -> Result<()> {
    dotenv::dotenv().ok();
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dotenv::var("GENERATORS")?)?;

    let generators: Vec<String> = serde_json::from_reader(file)?;
    let url = &generators[generator_type];
    let params = [("text", text), ("speaker", &voice_type.to_string())];
    let client = reqwest::Client::new();
    let voice_query_url = format!("{}/audio_query", url);
    let res = client.post(voice_query_url).query(&params).send().await?;
    let synthesis_body = res.text().await?;
    let synthesis_arg = [("speaker", voice_type)];
    let synthesis_url = format!("{}/synthesis", url);
    let synthesis_res = client
        .post(synthesis_url)
        .body(synthesis_body)
        .query(&synthesis_arg)
        .send()
        .await?;
    let _ = temp_file.write(&synthesis_res.bytes().await?)?;
    Ok(())
}
