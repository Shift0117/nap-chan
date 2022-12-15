use std::{convert::TryInto, fs::File, io::Write};

use crate::handler::Handler;
use anyhow::{anyhow, Result};
use reqwest;
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
    utils::{content_safe, ContentSafeOptions},
};
use tempfile;
use tracing::info;

use super::{db::UserConfigDB, text::TextMessage};

pub async fn play_voice(ctx: &Context, msg: Message, handler: &Handler) -> Result<()> {
    info!("{}", &msg.content);

    let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
    let clean_option = ContentSafeOptions::new();
    let user_id = msg.author.id.0 as i64;
    let nickname = handler
        .database
        .get_user_config_or_default(user_id)
        .await?
        .read_nickname
        .unwrap_or(
            msg.member
                .as_ref()
                .ok_or_else(|| anyhow!("member not found"))?
                .nick
                .as_ref()
                .unwrap_or(&msg.author.name)
                .to_string(),
        );
    let cleaned_content = content_safe(&ctx.cache, msg.content.clone(), &clean_option, &[])
        .make_read_text(&handler.database)
        .await;
    info!("{}", &cleaned_content);
    if cleaned_content.chars().all(|c| !c.is_alphanumeric()) {
        return Ok(());
    }
    let cleaned_text = format!(
        "{} {}",
        if msg.author.id != ctx.cache.as_ref().current_user_id() {
            nickname.make_read_text(&handler.database).await
        } else {
            String::new()
        },
        cleaned_content
    );

    let user_config = handler.database.get_user_config_or_default(user_id).await?;

    let voice_type = user_config.voice_type.try_into()?;
    let generator_type = user_config.generator_type;
    create_voice(
        &cleaned_text,
        voice_type,
        generator_type as usize,
        temp_file.as_file_mut(),
    )
    .await?;

    let guild = msg
        .guild(&ctx.cache)
        .ok_or_else(|| anyhow!("guild not found"))?;
    let guild_id = guild.id;
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
        if generator_type == 0 {
            track.set_volume(0.4);
        }
        handler.enqueue(track);
    }
    Ok(())
}


pub async fn play_voice_by_web_voicevox_api(ctx: &Context, msg: Message, handler: &Handler) -> Result<()> {
    info!("{}", &msg.content);

    let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
    let clean_option = ContentSafeOptions::new();
    let user_id = msg.author.id.0 as i64;
    let nickname = handler
        .database
        .get_user_config_or_default(user_id)
        .await?
        .read_nickname
        .unwrap_or(
            msg.member
                .as_ref()
                .ok_or_else(|| anyhow!("member not found"))?
                .nick
                .as_ref()
                .unwrap_or(&msg.author.name)
                .to_string(),
        );
    let cleaned_content = content_safe(&ctx.cache, msg.content.clone(), &clean_option, &[])
        .make_read_text(&handler.database)
        .await;
    info!("{}", &cleaned_content);
    if cleaned_content.chars().all(|c| !c.is_alphanumeric()) {
        return Ok(());
    }
    let cleaned_text = format!(
        "{} {}",
        if msg.author.id != ctx.cache.as_ref().current_user_id() {
            nickname.make_read_text(&handler.database).await
        } else {
            String::new()
        },
        cleaned_content
    );

    let user_config = handler.database.get_user_config_or_default(user_id).await?;

    let voice_type = user_config.voice_type.try_into()?;
    let generator_type = user_config.generator_type;
    create_voice_by_web_voicevox_api(
        &cleaned_text,
        voice_type,
        temp_file.as_file_mut(),
    )
    .await?;

    let guild = msg
        .guild(&ctx.cache)
        .ok_or_else(|| anyhow!("guild not found"))?;
    let guild_id = guild.id;
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
        if generator_type == 0 {
            track.set_volume(0.4);
        }
        handler.enqueue(track);
    }
    Ok(())
}

pub async fn create_voice_by_web_voicevox_api(
    text: &str,
    voice_type: u32,
    temp_file: &mut File,
) -> Result<()> {
    dotenv::dotenv().ok();
    let api_key = std::env::var("WEB_API_KEY")?;
    let url = "https://api.su-shiki.com/v2/voicevox/audio/".to_string();
    let params = [
        ("key", api_key.as_str()),
        ("text", text),
        ("speaker", &voice_type.to_string()),
    ];
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .query(&params)
        .send()
        .await?
        .bytes()
        .await?;
    let _ = temp_file.write(&res)?;
    Ok(())
}

pub async fn create_voice(
    text: &str,
    voice_type: u32,
    generator_type: usize,
    temp_file: &mut File,
) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("generators.json")?;

    let generators: Vec<String> = serde_json::from_reader(file)?;
    let url = generators[generator_type].clone();
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

pub async fn play_raw_voice(
    ctx: &Context,
    str: &str,
    voice_type: u32,
    generator_type: usize,
    guild_id: GuildId,
) -> Result<()> {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
    create_voice(str, voice_type, generator_type, temp_file.as_file_mut()).await?;
    let (_, path) = temp_file.keep()?;
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await?;
        source.metadata.source_url = Some(path.to_string_lossy().to_string());
        handler.enqueue_source(source);
    }
    Ok(())
}


pub async fn play_raw_voice_by_web_voicevox_api(
    ctx: &Context,
    str: &str,
    voice_type: u32,
    generator_type: usize,
    guild_id: GuildId,
) -> Result<()> {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp")?;
    create_voice_by_web_voicevox_api(str, voice_type,  temp_file.as_file_mut()).await?;
    let (_, path) = temp_file.keep()?;
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await?;
        source.metadata.source_url = Some(path.to_string_lossy().to_string());
        handler.enqueue_source(source);
    }
    Ok(())
}
