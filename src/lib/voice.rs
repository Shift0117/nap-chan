use std::{
    convert::TryInto,
    fs::{File, OpenOptions},
    io::Write,
};

use crate::Handler;

use super::text::Text;
use reqwest;
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
    utils::{content_safe, ContentSafeOptions},
};
use tempfile;
use tracing::info;

pub async fn play_voice(ctx: &Context, msg: Message, handler: &Handler) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    let clean_option = ContentSafeOptions::new();
    let user_id = msg.author.id.0 as i64;
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
    let cleaned = text.make_read_text(&handler).await;
    let q = sqlx::query!(
        "SELECT voice_type,generator_type FROM user_config WHERE user_id = ?",
        user_id
    )
    .fetch_one(&handler.user_config)
    .await;
    let (voice_type,generator_type) = 
    if let Ok(q) = q {
        let voice_type = q.voice_type.try_into().unwrap();
        let generator_type = q.generator_type.try_into().unwrap();
        create_voice(
            &cleaned.text,
            voice_type,
            generator_type,
            temp_file.as_file_mut(),
        )
        .await;
        (voice_type,generator_type)
    } else {
        create_voice(&cleaned.text, 1, 1, temp_file.as_file_mut()).await;
        (1,0)
    };
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
        let (mut track, _) = songbird::tracks::create_player(source);
        if generator_type == 0 {
            track.set_volume(0.64);
        }
        handler.enqueue(track);
        //handler.enqueue_source(source.into());
    }
}

pub async fn create_voice(text: &str, voice_type: u8, generator_type: u8, temp_file: &mut File) {
    dotenv::dotenv().ok();
    let base_url = std::env::var(match generator_type {
        0 => "BASE_URL_COEIRO",
        1 => "BASE_URL_VOICEVOX",
        _ => unreachable!(),
    })
    .expect("environment variable not found");
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

pub async fn play_raw_voice(
    ctx: &Context,
    str: &str,
    voice_type: u8,
    generator_type: u8,
    guild_id: GuildId,
) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    create_voice(str, voice_type, generator_type, temp_file.as_file_mut()).await;
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
