use std::io::Write;

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
    create_voice(&cleaned.text, &mut temp_file).await;
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

async fn create_voice(text: &str, temp_file: &mut NamedTempFile) {
    dotenv::dotenv().ok();
    let base_url = std::env::var("BASE_URL").expect("environment variable not found");
    let params = [("text", text), ("speaker", "5")];
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
    let synthesis_arg = [("speaker", 5i16)];
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

pub async fn play_raw_voice(ctx: &Context, str: &str, guild_id: GuildId) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    create_voice(str, &mut temp_file).await;
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
