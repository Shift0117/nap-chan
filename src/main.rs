mod commands;
mod handler;
mod lib;
use dotenv::dotenv;
use serenity::client::ClientBuilder;
use serenity::http::Http;
use serenity::{async_trait, framework::StandardFramework};
use songbird::{Event, EventContext, SerenityInit};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::handler::Handler;
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::from_str(&database_url).unwrap())
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");
    let _ = database.insert_speaker_data().await;
    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("environment variable not found");
    let framework = StandardFramework::new();
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
