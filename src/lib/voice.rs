use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use super::text::Text;
use reqwest;
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
    utils::{content_safe, ContentSafeOptions},
};
use tempfile::{self, NamedTempFile};

pub async fn play_voice(ctx: &Context, msg: Message) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    let clean_option = ContentSafeOptions::new();
    let text = Text::new(format!(
        "{} {}",
        if msg.author.id != ctx.cache.as_ref().current_user_id().await {
            match &msg.member.as_ref().expect("member not found?").nick {
                Some(nick) => nick,
                None => &msg.author.name,
            }
        } else {
            ""
        },
        content_safe(&ctx.cache, msg.content.clone(), &clean_option).await
    ));
    let cleaned = text.make_read_text(&ctx).await;
    create_voice(&cleaned.text, 5, temp_file.as_file_mut()).await;
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let (_, path) = temp_file.keep().unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await.unwrap();
        source.metadata.source_url = Some(path.to_string_lossy().to_string());
        handler.enqueue_source(source.into());
    }
}

pub async fn create_voice(text: &str, voice_type: u8, temp_file: &mut File) {
    dotenv::dotenv().ok();
    let base_url = std::env::var("BASE_URL").expect("environment variable not found");
    let params = [("text", text), ("speaker", &voice_type.to_string())];
    let client = reqwest::Client::new();
    let voice_query_url = format!("{}/audio_query", base_url);
    let res = client
        .post(voice_query_url)
        .query(&params)
        .send()
        .await
        .expect("Panic in audio query");
    println!("{}", res.status());
    let synthesis_body = res.text().await.expect("Panic in get body");
    let synthesis_arg = [("speaker", voice_type)];
    let synthesis_url = format!("{}/synthesis", base_url);
    let synthesis_res = client
        .post(synthesis_url)
        .body(synthesis_body)
        .query(&synthesis_arg)
        .send()
        .await
        .expect("Panic in synthesis query");
    dbg!(&synthesis_res.status());
    temp_file
        .write(&synthesis_res.bytes().await.unwrap())
        .unwrap();
}

pub async fn play_raw_voice(ctx: &Context, str: &str, voice_type: u8, guild_id: GuildId) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    create_voice(str, voice_type, temp_file.as_file_mut()).await;
    let (_, path) = temp_file.keep().unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await.unwrap();
        source.metadata.source_url = Some(path.to_string_lossy().to_string());
        handler.enqueue_source(source.into());
    }
}

pub async fn create_sample_voices() {
    std::fs::create_dir("temp").ok();
    for i in 0..=5 {
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(format!("sample_voice/sample_{}", i))
        {
            create_voice(&format!("タイプ{}わこんな感じだよ", i), i, &mut file).await;
        }
    }
}
