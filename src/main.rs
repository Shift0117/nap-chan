mod commands;
mod lib;
use commands::meta;
use dotenv::dotenv;
use lib::voice::*;
use serenity::client::{ClientBuilder, Context};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::http::Http;
use serenity::model::id::{GuildId, UserId};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::{application_command, Interaction, InteractionResponseType};
use serenity::model::prelude::VoiceState;
use serenity::{
    async_trait,
    client::EventHandler,
    framework::StandardFramework,
    model::{channel::Message, gateway::Ready},
};
use songbird::{Event, EventContext, SerenityInit};
use sqlx::query;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Handler {
    user_config: sqlx::SqlitePool,
    dict: sqlx::SqlitePool,
    read_channel_id: Arc<Mutex<Option<serenity::model::id::ChannelId>>>,
}
const GUILD_IDS_PATH: &str = "guilds.json";

type SlashCommandResult = Result<String, String>;

impl Handler {
    pub async fn hello(
        &self,
        command: &ApplicationCommandInteraction,
        greet: &str,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;

        sqlx::query!(
            "INSERT OR REPLACE INTO user_config (user_id,hello) VALUES (?,?)",
            user_id,
            greet
        )
        .execute(&self.user_config)
        .await
        .ok();

        Ok(format!(
            "{}さん、これから{}ってあいさつするね",
            command.member.as_ref().unwrap().user.name,
            greet
        ))
    }
    pub async fn bye(
        &self,
        command: &ApplicationCommandInteraction,
        greet: &str,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
        sqlx::query!(
            "INSERT OR REPLACE INTO user_config (user_id,bye) VALUES (?,?)",
            user_id,
            greet
        )
        .execute(&self.user_config)
        .await
        .ok();

        Ok(format!(
            "{}さん、これから{}ってあいさつするね",
            command.member.as_ref().unwrap().user.name,
            greet
        ))
    }
    pub async fn set_voice_type(
        &self,
        command: &ApplicationCommandInteraction,
        voice_type: i64,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
        sqlx::query!(
            "INSERT OR REPLACE INTO user_config (user_id,voice_type) VALUES (?,?)",
            user_id,
            voice_type
        )
        .execute(&self.user_config)
        .await
        .ok();
        Ok(format!("ボイスタイプを{}に変えたよ", voice_type).to_string())
    }
    pub async fn add(&self, before: &str, after: &str) -> SlashCommandResult {
        sqlx::query!(
            "INSERT OR REPLACE INTO dict (word,read_word) VALUES (?,?)",
            before,
            after
        )
        .execute(&self.dict)
        .await
        .ok();
        Ok(format!("これからは、{}を{}って読むね", before, after))
    }
    pub async fn rem(&self, word: &str) -> SlashCommandResult {
        if let Ok(_) = sqlx::query!("DELETE FROM dict WHERE word = ?", word)
            .execute(&self.dict)
            .await
        {
            Ok(format!("これからは{}って読むね", word))
        } else {
            Err("その単語は登録されてないよ！".to_string())
        }
    }
}

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
                            .name("bye")
                            .description("出た時のあいさつを変えます")
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
            let user_name = &new.member.as_ref()?.nick.as_ref()?;

            let greeting_type = if let Some(ref old) = old {
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
            let uid = user_id.0 as i64;
            sqlx::query!(
                "INSERT OR IGNORE INTO user_config (user_id) VALUES (?)",
                uid
            )
            .execute(&self.user_config)
            .await
            .ok();
            let q = sqlx::query!(
                "SELECT hello,bye,voice_type FROM user_config WHERE user_id = ?",
                uid
            )
            .fetch_one(&self.user_config)
            .await
            .unwrap();
            let greet_text = match greeting_type {
                0 => q.hello,
                1 => q.bye,
                _ => unreachable!(),
            };
            let text = lib::text::Text::new(format!("{}さん、{}", user_name, greet_text))
                .make_read_text(&self)
                .await;
            let voice_type = q.voice_type.try_into().unwrap();
            play_raw_voice(&ctx, &text.text, voice_type, guild_id?).await;
            Some(())
        }
        .await;
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let nako_id = ctx.cache.current_user_id().await;
        let voice_channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_states| voice_states.channel_id);
        let text_channel_id = msg.channel_id;

        let uid = msg.author.id.0 as i64;
        let read_channel_id = self.read_channel_id.lock().await.clone();
        if read_channel_id == Some(text_channel_id) {
            if let Some(voice_channel_id) = voice_channel_id {
                let members = ctx
                    .cache
                    .channel(voice_channel_id)
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
                let q = query!("SELECT voice_type FROM user_config WHERE user_id = ?", uid)
                    .fetch_one(&self.user_config)
                    .await;

                let voice_type = q
                    .and_then(|q| Ok(q.voice_type.try_into().unwrap()))
                    .unwrap_or(1);
                if members.contains(&nako_id) && msg.author.id != nako_id {
                    dbg!(&msg);
                    play_voice(&ctx, msg, voice_type, self).await;
                };
            }
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let mut voice_type = 1;
            let content = match command.data.name.as_str() {
                "join" => meta::join(&ctx, &command, &self.read_channel_id).await,
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
                        self.add(before, after).await
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
                        self.rem(word).await
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
                        //dict::hello(&ctx,&command,&greet).await
                        self.hello(&command, &greet).await
                    } else {
                        unreachable!()
                    }
                }
                "bye" => {
                    let greet = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    if let application_command::ApplicationCommandInteractionDataOptionValue::String(greet) = greet {
                        self.bye(&command,&greet).await
                    } else {
                        unreachable!()
                    }
                }
                "play_sample_voice" => {
                    let voice_type_args = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected integer")
                        .resolved
                        .as_ref()
                        .expect("Expected integer");

                    if let application_command::ApplicationCommandInteractionDataOptionValue::Integer(
                            voice_type_args,
                        )

                     = voice_type_args
                    {
                        voice_type = *voice_type_args as u8;
                        commands::voice::play_sample_voice(
                            &ctx,
                            command.guild_id.unwrap(),
                            voice_type,
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
                        self.set_voice_type(&command, *voice_type).await
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
                            message.content(match content.clone() {
                                Ok(content) => content,
                                Err(error) => format!("エラー: {}", error).to_string(),
                            })
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            } else {
                if let Ok(content) = content {
                    play_raw_voice(&ctx, &content, voice_type, command.guild_id.unwrap()).await;
                }
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
    let user_config = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("user_config.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("./migrations")
        .run(&user_config)
        .await
        .expect("Couldn't run database migrations");
    let dict = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("dict.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("./migrations")
        .run(&dict)
        .await
        .expect("Couldn't run database migrations");

    let application_id = std::env::var("APP_ID").unwrap().parse().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("environment variable not found");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .group(&GENERAL_GROUP);
    let mut client =
        ClientBuilder::new_with_http(Http::new_with_token_application_id(&token, application_id))
            .event_handler(Handler {
                user_config,
                dict,
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
