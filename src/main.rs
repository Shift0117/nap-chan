mod commands;
mod lib;
use commands::{dict, meta};
use dotenv::dotenv;
use lib::voice::*;
use serenity::client::{ClientBuilder, Context};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::http::Http;
use serenity::model::id::GuildId;
use serenity::model::interactions::{application_command, Interaction, InteractionResponseType};
use serenity::model::prelude::VoiceState;
use serenity::{
    async_trait,
    client::EventHandler,
    framework::StandardFramework,
    model::{channel::Message, gateway::Ready},
};
use songbird::{Event, EventContext, SerenityInit};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::commands::dict::{generate_dictonaries, DictHandler};
struct Handler;
const GUILD_IDS_PATH: &str = "guilds.json";

const GREETING: [(&str, &str); 2] = [("hello", "こんにちは"), ("bye", "ばいばい")];

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);
        let guilds_file = if let Ok(file) = File::open(GUILD_IDS_PATH) {
            file
        } else {
            let mut tmp = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(GUILD_IDS_PATH)
                .expect("File creation error");
            tmp.write_all("[]".as_bytes()).ok();
            tmp.seek(std::io::SeekFrom::Start(0)).ok();
            tmp
        };
        let reader = std::io::BufReader::new(guilds_file);
        let guild_ids: HashSet<GuildId> =
            serde_json::from_reader(reader).expect("JSON parse error");
        tracing::info!("{:?}", &guild_ids);

        create_sample_voices().await;

        /*let old_global_commands = ctx.http.get_global_application_commands().await.unwrap();
        for command in old_global_commands {
            dbg!(command.name);
            ctx.http.delete_global_application_command(command.id.0).await;
        }*/
        for guild_id in guild_ids {
            /*let old_commands = guild_id.get_application_commands(&ctx.http).await.unwrap();
            for command in old_commands {
                dbg!(command.name);
                guild_id
                    .delete_application_command(&ctx.http, command.id)
                    .await
                    .ok();
            }*/
            let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("join")
                            .description("なこちゃんに来てもらいます")
                    })
                    .create_application_command(|command| {
                        command
                            .name("leave")
                            .description("なこちゃんとばいばいします")
                    })
                    .create_application_command(|command| {
                        command
                            .name("add")
                            .create_option(|option| {
                                option
                                    .kind(application_command::ApplicationCommandOptionType::String)
                                    .required(true)
                                    .name("before")
                                    .description("string")
                            })
                            .create_option(|option| {
                                option
                                    .kind(application_command::ApplicationCommandOptionType::String)
                                    .required(true)
                                    .description("string")
                                    .name("after")
                            })
                            .description("before を after と読むようにします")
                    })
                    .create_application_command(|command| {
                        command
                            .name("rem")
                            .create_option(|option| {
                                option
                                    .kind(application_command::ApplicationCommandOptionType::String)
                                    .required(true)
                                    .name("word")
                                    .description("string")
                            })
                            .description("word の読み方を忘れます")
                    })
                    .create_application_command(|command| {
                        command
                            .name("mute")
                            .description("なこちゃんをミュートします")
                    })
                    .create_application_command(|command| {
                        command
                            .name("unmute")
                            .description("なこちゃんのミュートを解除します")
                    })
                    .create_application_command(|command| {
                        command
                            .name("hello")
                            .description("入った時のあいさつを変えます")
                            .create_option(|option| {
                                option
                                    .kind(application_command::ApplicationCommandOptionType::String)
                                    .required(true)
                                    .name("greet")
                                    .description("string")
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("play_sample_voice")
                            .description("入力されたタイプのサンプルボイスを再生します")
                            .create_option(|option| {
                                option
                                    .kind(
                                        application_command::ApplicationCommandOptionType::Integer,
                                    )
                                    .max_int_value(5)
                                    .min_int_value(0)
                                    .required(true)
                                    .name("type")
                                    .description("0 から 5 の整数値")
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("set_voice_type")
                            .description("ボイスタイプを変えます")
                            .create_option(|option| {
                                option
                                    .kind(
                                        application_command::ApplicationCommandOptionType::Integer,
                                    )
                                    .max_int_value(5)
                                    .min_int_value(0)
                                    .required(true)
                                    .name("type")
                                    .description("0 から 5 の整数値")
                            })
                    })
            })
            .await;
        }
    }
    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let nako_id = &ctx.cache.current_user_id().await;
        let _ = async move {
            let nako_channel_id = guild_id?
                .to_guild_cached(&ctx.cache)
                .await?
                .voice_states
                .get(&nako_id)?
                .channel_id?;
            let channel_id = guild_id?
                .to_guild_cached(&ctx.cache)
                .await?
                .voice_states
                .get(nako_id)?
                .channel_id?;
            let members_count = ctx
                .cache
                .channel(channel_id)
                .await?
                .guild()?
                .members(&ctx.cache)
                .await
                .ok()?
                .iter()
                .filter(|member| member.user.id.0 != nako_id.0)
                .count();
            if members_count == 0 {
                meta::leave(&ctx, guild_id?).await.ok();
                return Some(());
            }
            let user_id = new.user_id;
            if nako_id.0 == user_id.0 {
                return Some(());
            }
            let user_name = &new.member.as_ref()?.user.name;
            let dicts_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<DictHandler>()?.clone()
            };
            let greeting_index = if let Some(ref old) = old {
                if old.self_mute != new.self_mute
                    || old.self_deaf != new.self_deaf
                    || old.self_video != new.self_video
                    || old.self_stream != new.self_stream
                {
                    return Some(());
                }
                if old.channel_id == Some(nako_channel_id) {
                    1
                } else {
                    0
                }
            } else {
                0
            };
            let greet_text = dicts_lock
                .lock()
                .await
                .get_greeting(&user_id, GREETING[greeting_index].0)
                .unwrap_or_else(|| GREETING[greeting_index].1.to_string());
            tracing::info!("{:?}", dicts_lock.lock().await);
            let text = lib::text::Text::new(format!("{}さん、{}", user_name, greet_text))
                .make_read_text(&ctx)
                .await;
            play_raw_voice(&ctx, &text.text, 1, guild_id?).await;
            Some(())
        }
        .await;
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let nako_id = ctx.cache.current_user_id().await;
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .unwrap()
            .channel_id
            .unwrap();
        let members = ctx
            .cache
            .channel(channel_id)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .members(&ctx.cache)
            .await
            .unwrap()
            .iter()
            .map(|member| member.user.id)
            .collect::<Vec<_>>();
        let voice_type = *ctx
            .data
            .read()
            .await
            .get::<DictHandler>()
            .unwrap()
            .lock()
            .await
            .voice_type_dict
            .get(&msg.author.id)
            .unwrap_or(&1);

        if members.contains(&nako_id) && msg.author.id != nako_id {
            dbg!(&msg);
            play_voice(&ctx, msg, voice_type).await;
        };
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let content = match command.data.name.as_str() {
                "join" => meta::join(&ctx, &command).await,
                "leave" => meta::leave(&ctx, command.guild_id.unwrap()).await,
                "add" => {
                    let options = &command.data.options;
                    let before = options
                        .get(0)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    let after = options
                        .get(1)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    if let (
                        application_command::ApplicationCommandInteractionDataOptionValue::String(
                            before,
                        ),
                        application_command::ApplicationCommandInteractionDataOptionValue::String(
                            after,
                        ),
                    ) = (before, after)
                    {
                        dict::add(&ctx, &command, before, after).await
                    } else {
                        unreachable!()
                    }
                }
                "rem" => {
                    let word = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    if let application_command::ApplicationCommandInteractionDataOptionValue::String(word) = word {
                        dict::rem(&ctx,&command,word).await
                    }
                    else {
                        unreachable!()
                    }
                }
                "mute" => meta::mute(&ctx, &command).await,
                "unmute" => meta::unmute(&ctx, &command).await,
                "hello" => {
                    let greet = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    if let application_command::ApplicationCommandInteractionDataOptionValue::String(greet) = greet {
                        dict::hello(&ctx,&command,&greet).await
                    } else {
                        unreachable!()
                    }
                }
                "play_sample_voice" => {
                    let voice_type = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected integer")
                        .resolved
                        .as_ref()
                        .expect("Expected integer");

                    if let application_command::ApplicationCommandInteractionDataOptionValue::Integer(
                            voice_type,
                        )

                     = voice_type
                    {

                        commands::voice::play_sample_voice(
                            &ctx,
                            command.guild_id.unwrap(),
                            *voice_type as u8,
                        )
                        .await
                    } else {
                        unreachable!()
                    }
                }
                "set_voice_type" => {
                    let voice_type = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected integer")
                        .resolved
                        .as_ref()
                        .expect("Expected integer");

                    if let application_command::ApplicationCommandInteractionDataOptionValue::Integer(
                            voice_type,
                        )
                     = voice_type {
                        dict::set_voice_type(&ctx, &command, *voice_type as u8).await
                     } else {
                        unreachable!()
                     }
                }
                _ => Err("未実装だよ！".to_string()),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(match content {
                                Ok(content) => content,
                                Err(error) => format!("エラー: {}", error).to_string(),
                            })
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
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
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
    tracing::info!("register called");
    let guild_id = msg.guild_id.unwrap();
    let mut guilds_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(GUILD_IDS_PATH)
        .unwrap();
    let reader = std::io::BufReader::new(&guilds_file);
    let mut guild_ids: Vec<GuildId> = serde_json::from_reader(reader).expect("JSON parse error");
    guilds_file.seek(io::SeekFrom::Start(0)).ok();

    guild_ids.push(guild_id);
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

    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("environment variable not found");
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
        data.insert::<DictHandler>(Arc::new(Mutex::new(generate_dictonaries())));
    }
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
