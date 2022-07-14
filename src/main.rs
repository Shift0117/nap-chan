mod commands;
mod handler;
mod lib;
use dotenv::dotenv;
use serenity::client::{ClientBuilder, Context};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::http::Http;
use serenity::model::id::GuildId;
use serenity::{async_trait, framework::StandardFramework, model::channel::Message};
use songbird::{Event, EventContext, SerenityInit};
use std::collections::HashSet;
use std::io::{self, Seek, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::handler::{Handler, GUILD_IDS_PATH};
use crate::lib::db::SpeakerDB;

#[derive(Debug)]
pub struct Dict {
    word: String,
    read_word: String,
}

struct TrackEndNotifier;

#[async_trait]
impl songbird::EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (_, handle) in track_list.iter() {
                let path = handle.metadata().source_url.as_ref().unwrap();
                tracing::info!("played file path: {:?}", path);
                if !path.ends_with(".wav") {
                    std::fs::remove_file(Path::new(handle.metadata().source_url.as_ref().unwrap()))
                        .unwrap();
                }
            }
        }
        None
    }
}

#[group]
#[commands(register)]
struct General;

#[command]
#[only_in(guilds)]
async fn register(_ctx: &Context, msg: &Message) -> CommandResult {
    tracing::info!("register called");
    let guild_id = msg.guild_id.unwrap();
    let mut guilds_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(GUILD_IDS_PATH)
        .unwrap();
    let reader = std::io::BufReader::new(&guilds_file);
    let mut guild_ids: HashSet<GuildId> =
        serde_json::from_reader(reader).expect("JSON parse error");
    guilds_file.seek(io::SeekFrom::Start(0)).ok();
    guild_ids.insert(guild_id);
    let guild_ids_json = serde_json::to_string(&guild_ids).unwrap();
    guilds_file.write_all(guild_ids_json.as_bytes()).ok();
    tracing::info!("register finished");
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv().ok();
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");
    database.generate_speaker_db().await.unwrap();
    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("environment variable not found");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .group(&GENERAL_GROUP);
    let mut client =
        ClientBuilder::new_with_http(Http::new_with_token_application_id(&token, application_id))
            .event_handler(Handler {
                database,
                read_channel_id: Arc::new(Mutex::new(None)),
            })
            .framework(framework)
            .register_songbird()
            .await
            .expect("Err creating client");
    std::fs::create_dir("temp").ok();

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| tracing::info!("Client ended: {:?}", why));
    });
    tokio::signal::ctrl_c().await.unwrap();
    std::fs::remove_dir_all("temp").unwrap();
    std::fs::create_dir("temp").unwrap();
    tracing::info!("Ctrl-C received, shutting down...");
}
