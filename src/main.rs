mod commands;
mod lib;
use commands::{dict, meta};
use dotenv::dotenv;
use lib::voice::*;
use serenity::client::{ClientBuilder, Context};
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
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::lib::text::{DictHandler, DICT_PATH};

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
        })
        .await;
        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
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
        let guild = msg.guild(&ctx.cache).await.unwrap();
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
        if members.contains(&ctx.cache.current_user_id().await) {
            play_voice(&ctx, msg).await;
        };
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let content = match command.data.name.as_str() {
                "join" => {
                    meta::join(&ctx, &command).await
                }
                "leave" => {
                    meta::leave(&ctx, &command).await
                }
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
                "mute" => {
                    meta::mute(&ctx, &command).await
                }
                "unmute" => {
                    meta::unmute(&ctx, &command).await
                }
                _ => Err("未実装だよ！".to_string()),
            };
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(match content {
                            Ok(content) => content,
                            Err(error) => format!("エラー: {}",error).to_string()
                        }))
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
                std::fs::remove_file(Path::new(handle.metadata().source_url.as_ref().unwrap()))
                    .unwrap();
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
    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("VOICEVOX_TOKEN").expect("environment variable not found");
    let dict_file = std::fs::File::open(DICT_PATH).unwrap();
    let reader = std::io::BufReader::new(dict_file);
    let dict: HashMap<String, String> = serde_json::from_reader(reader).unwrap();
    let framework = StandardFramework::new().configure(|c| c.prefix(">"));
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
    std::fs::remove_dir_all("temp").unwrap();
    std::fs::create_dir("temp").unwrap();
    tracing::info!("Ctrl-C received, shutting down...");
}