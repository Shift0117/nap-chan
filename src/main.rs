mod commands;
mod handler;

mod lib;
pub mod listener;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

use songbird::{Event, EventContext, SerenityInit};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Dict {
    word: String,
    read_word: String,
}

struct TrackEndNotifier;
pub struct Data {
    pub database: sqlx::SqlitePool,
    pub read_channel_id: Arc<Mutex<Option<serenity::model::id::ChannelId>>>,
    pub voice_types: Arc<Mutex<Vec<lib::db::VoiceType>>>,
}

#[poise::async_trait]
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
#[poise::command(slash_command)]
async fn connect(ctx: Context<'_>) -> anyhow::Result<()> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;
    let channel_id = ctx
        .guild()
        .unwrap()
        .voice_states
        .get(&author_id)
        .unwrap()
        .channel_id
        .unwrap();

    let manager = songbird::get(ctx.discord())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (_, err) = manager.join(guild_id, channel_id).await;
    ctx.say(format!("{:?}", &err)).await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
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
    let voice_types = lib::db::get_voice_types()
        .await
        .expect("Couldn't get voice types");
    let _application_id: String = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("environment variable not found");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                register(),
                commands::meta::join(),
                commands::meta::leave(),
                commands::meta::mute(),
                commands::meta::unmute(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            listener: |ctx, event, framework, user_data| {
                Box::pin(listener::event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .token(token)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    database,
                    read_channel_id: Arc::new(Mutex::new(None)),
                    voice_types: Arc::new(Mutex::new(voice_types)),
                })
            })
        })
        .client_settings(songbird::register);

    framework.run().await.unwrap();
    // let framework = StandardFramework::new();
    // let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    // let mut client = Client::builder(&token, intents)
    //     .event_handler(Handler {
    //         database,
    //         read_channel_id: Arc::new(Mutex::new(None)),
    //         voice_types: Arc::new(Mutex::new(voice_types)),
    //     })
    //     .framework(framework)
    //     .register_songbird()
    //     .await
    //     .expect("Err creating client");
    // std::fs::create_dir("temp").ok();

    // tokio::spawn(async move {
    //     let _ = client
    //         .start()
    //         .await
    //         .map_err(|why| tracing::info!("Client ended: {:?}", why));
    // });
    // tokio::signal::ctrl_c().await.unwrap();
    // std::fs::remove_dir_all("temp").unwrap();
    // std::fs::create_dir("temp").unwrap();
    // tracing::info!("Ctrl-C received, shutting down...");
}
