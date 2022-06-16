mod lib;
use serde_json::to_string;
use serenity::http::Http;
use serenity::model::id::GuildId;
use serenity::model::interactions::{application_command, Interaction, InteractionResponseType};
use serenity::model::prelude::VoiceState;
use serenity::prelude::TypeMapKey;
use songbird::{Event, EventContext, SerenityInit, TrackEvent};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use dotenv::dotenv;
use lib::voice::*;
use serenity::client::{ClientBuilder, Context};
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
};

const DICT_PATH: &str = "read_dict.json";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);
        dotenv().ok();

        let guild_id = GuildId(
            std::env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command
                        .name("join")
                        .description("なこちゃんに来てもらいます")
                })
        })
        .await;
        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
        let guild_command = serenity::model::interactions::application_command::ApplicationCommand::
        create_global_application_command(&ctx.http, |command| {
            command.name("wonderful_command").description("An amazing command")
        })
        
        .await;
        println!(
            "I created the following global slash command: {:#?}",
            guild_command
        );
    }
    async fn voice_state_update(
        &self,
        _ctx: Context,
        _: Option<GuildId>,
        _old: Option<VoiceState>,
        _new: VoiceState,
    ) {
        tracing::info!("{:?}\n{:?}", _old, _new);
        tracing::info!("{} is connected!", _new.member.unwrap().user.name);
    }
    async fn message(&self, ctx: Context, msg: Message) {
        dbg!(&msg.guild_id);
        
        let guild = msg.guild(&ctx.cache).await.unwrap();
        
        if guild
            .members
            .contains_key(&ctx.cache.current_user_id().await) && false
        {
            play_voice(&ctx, msg).await;
        };
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "join" => {
                    let guild_id = command.guild_id.unwrap();
                    let author_id = command.member.as_ref().unwrap().user.id;
                    let channel_id =command.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap().voice_states.get(&author_id)
                    .and_then(|voice_state| voice_state.channel_id).unwrap();
                    let connect_to = channel_id;
                    let manager = songbird::get(&ctx)
                        .await
                        .expect("Songbird Voice client placed in at initialisation.")
                        .clone();                    
                    let (handle_lock, _) = manager.join(guild_id, connect_to).await;
                    let mut handle = handle_lock.lock().await;
                    handle.deafen(true).await.unwrap();
                    handle.add_global_event(Event::Track(TrackEvent::End), TrackEndNotifier);
                    "こんにちは".to_string()
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[group]
#[commands(join, leave, mute, unmute, add)]
struct General;

struct TrackEndNotifier;

#[async_trait]
impl songbird::EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (_, handle) in track_list.iter() {
                std::fs::remove_file(Path::new(handle.metadata().source_url.as_ref().unwrap()))
                    .unwrap();
            }
        }
        None
    }
}

struct DictHandler;

impl TypeMapKey for DictHandler {
    type Value = Arc<Mutex<HashMap<String, String>>>;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv().ok();
    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("VOICEVOX_TOKEN").expect("environment variable not found");
    let dict_file = std::fs::File::open(DICT_PATH).unwrap();
    let reader = std::io::BufReader::new(dict_file);
    let dict: HashMap<String, String> = serde_json::from_reader(reader).unwrap();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .group(&GENERAL_GROUP);
    let mut client =
        ClientBuilder::new_with_http(Http::new_with_token_application_id(&token, application_id))
            .event_handler(Handler)
            .framework(framework)
            .register_songbird()
            .await
            .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<DictHandler>(Arc::new(Mutex::new(dict)));
    }
    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| tracing::info!("Client ended: {:?}", why));
    });

    tokio::signal::ctrl_c().await.unwrap();
    tracing::info!("Ctrl-C received, shutting down...");
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handle_lock, _) = manager.join(guild_id, connect_to).await;
    let mut handle = handle_lock.lock().await;
    handle.deafen(true).await.unwrap();
    handle.add_global_event(Event::Track(TrackEvent::End), TrackEndNotifier);
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await?;
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;
            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    let content = if handler.is_mute() {
        "Already muted".to_string()
    } else {
        if let Err(e) = handler.mute(true).await {
            format!("Failed: {:?}", e)
        } else {
            "Now muted".to_string()
        }
    };
    msg.channel_id.say(&ctx.http, content).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let content = if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            format!("Failed: {:?}", e)
        } else {
            "Unmuted".to_string()
        }
    } else {
        "Not in a voice channel to unmute in".to_string()
    };
    msg.channel_id.say(&ctx.http, content).await?;
    Ok(())
}

#[command]
#[only_in(guild)]
#[num_args(2)]
async fn add(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let before: String = args.single().unwrap();
    let after: String = args.single().unwrap();
    dbg!(&before, &after);
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dict = dict_lock.lock().await;
    dict.insert(before, after);
    let dict = dict.clone();
    let dict_json = to_string(&dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(DICT_PATH)
        .unwrap();
    dict_file.write_all(dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();

    Ok(())
}

#[command]
#[only_in(guild)]
#[num_args(1)]
async fn rem(ctx: &Context, _: &Message, mut args: Args) -> CommandResult {
    let before: String = args.single().unwrap();
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dict = dict_lock.lock().await;
    if dict.contains_key(&before) {
        dict.remove(&before);
    }
    let dict = dict.clone();
    let dict_json = to_string(&dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open("read_dict.json")
        .unwrap();
    dict_file.write_all(dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();
    Ok(())
}
