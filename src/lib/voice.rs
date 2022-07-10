use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::{self, File},
    io::Write,
    sync::RwLock,
};

use reqwest;
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
    utils::{content_safe, ContentSafeOptions},
};
use tempfile;
use tracing::info;

use crate::handler::{Generators, Handler, Speaker};

use super::{db::UserConfigDB, text::TextMessage};

//pub static SPEAKER_BIJECTION: Lazy<SpeakerBijection> = Lazy::new(SpeakerBijection::new);
#[derive(Hash)]
pub struct SpeakerId {
    id: u8,
    generator: Generators,
}

#[derive(Default)]
pub struct SpeakerBijection {
    name_to_id: RwLock<HashMap<String, SpeakerId>>,
    id_to_name: RwLock<HashMap<SpeakerId, String>>,
}

impl SpeakerBijection {
    pub async fn new() -> Self {
        let voice_types = get_speaker_data().await;

        todo!()
    }
}

pub async fn play_voice(ctx: &Context, msg: Message, handler: &Handler) {
    let mut temp_file = tempfile::Builder::new().tempfile_in("temp").unwrap();
    let clean_option = ContentSafeOptions::new();
    let user_id = msg.author.id.0 as i64;
    let nickname = handler
        .database
        .get_user_config_or_default(user_id)
        .await
        .read_nickname
        .unwrap_or(
            msg.member
                .as_ref()
                .expect("member not found")
                .nick
                .as_ref()
                .unwrap_or(&msg.author.name)
                .to_string(),
        );
    let cleaned_content = content_safe(&ctx.cache, msg.content.clone(), &clean_option)
        .await
        .make_read_text(&handler.database)
        .await;
        info!("{}",&cleaned_content);
    if cleaned_content.chars().all(|c| !c.is_alphanumeric()) {
        return ();
    }
    let cleaned_text = format!(
        "{} {}",
        if msg.author.id != ctx.cache.as_ref().current_user_id().await {
            nickname.make_read_text(&handler.database).await
        } else {
            String::new()
        },
        cleaned_content
    );

    let user_config = handler.database.get_user_config_or_default(user_id).await;

    let voice_type = user_config.voice_type.try_into().unwrap();
    let generator_type = user_config.generator_type.try_into().unwrap();
    create_voice(
        &cleaned_text,
        voice_type,
        generator_type,
        temp_file.as_file_mut(),
    )
    .await;

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
            track.set_volume(0.4);
        }
        handler.enqueue(track);
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

pub async fn get_speaker_data() -> [Vec<Speaker>; 2] {
    dotenv::dotenv().ok();
    fs::create_dir("speakers").ok();
    let voicevox_file = match File::open("speakers/voicevox.json") {
        Ok(file) => file,
        Err(_) => {
            let base_url =
                std::env::var("BASE_URL_VOICEVOX").expect("environment variable not found");
            let query_url = format!("{}/speaker", base_url);
            let client = reqwest::Client::new();
            let res = client
                .get(query_url)
                .send()
                .await
                .expect("Panic in speaker info query");
            let mut file = File::create("speakers/voicevox.json").unwrap();
            file.write(&res.bytes().await.unwrap()).ok();
            file
        }
    };
    let reader = std::io::BufReader::new(voicevox_file);
    let voicevox_voice_types = serde_json::from_reader::<_, Vec<Speaker>>(reader).unwrap();
    let coeiro_file = match File::open("speakers/coeiro.json") {
        Ok(file) => file,
        Err(_) => {
            let base_url =
                std::env::var("BASE_URL_COEIROINK").expect("environment variable not found");
            let query_url = format!("{}/speaker", base_url);
            let client = reqwest::Client::new();
            let res = client
                .get(query_url)
                .send()
                .await
                .expect("Panic in speaker info query");
            let mut file = File::create("speakers/coeiro.json").unwrap();
            file.write(&res.bytes().await.unwrap()).ok();
            file
        }
    };
    let reader = std::io::BufReader::new(coeiro_file);
    let coeiro_voice_types = serde_json::from_reader::<_, Vec<Speaker>>(reader).unwrap();
    [coeiro_voice_types, voicevox_voice_types]
}

