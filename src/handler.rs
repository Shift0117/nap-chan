use anyhow::{anyhow, Result};
use serde::Deserialize;
use serenity::{
    async_trait,
    builder::CreateSelectMenu,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        interactions::{
            application_command::{
                self, ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
            },
            message_component::ComponentType,
            Interaction, InteractionResponseType,
        },
        prelude::{Ready, VoiceState},
    },
};
use sqlx::query;
use std::{
    collections::HashSet,
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{Seek, Write},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    commands::{
        definition,
        interactions::{get_display_name, interaction_create_with_text},
        meta, util,
    },
    lib::{
        db::{DictDB, UserConfigDB,SpeakerDB, VoiceType},
        text::TextMessage,
        voice::{play_raw_voice, play_voice},
    },
};
pub const GUILD_IDS_PATH: &str = "guilds.json";

#[derive(Clone, Copy, Hash)]
pub enum Generators {
    COEIROINK = 0,
    VOICEVOX = 1,
}
impl TryFrom<&str> for Generators {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "COEIROINK" => Ok(Self::COEIROINK),
            "VOICEVOX" => Ok(Self::VOICEVOX),
            _ => Err(anyhow!("no such generator_type")),
        }
    }
}

impl TryFrom<u8> for Generators {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::COEIROINK),
            1 => Ok(Self::VOICEVOX),
            _ => Err(anyhow!("no such generator_type"))
        }
    }
}

impl Into<&str> for Generators {
    fn into(self) -> &'static str {
        match self {
            Self::COEIROINK => "COEIROINK",
            Self::VOICEVOX => "VOICEVOX",
        }
    }
}

pub struct Handler {
    pub database: sqlx::SqlitePool,
    pub read_channel_id: Arc<Mutex<Option<serenity::model::id::ChannelId>>>,
}
pub type Command = ApplicationCommandInteraction;
pub type ArgumentValue = ApplicationCommandInteractionDataOptionValue;
#[derive(Clone)]
pub struct SlashCommandTextResult {
    msg: String,
    read: bool,
    format: bool,
    voice_type: Option<u8>,
    generator_type: Option<u8>,
}

impl SlashCommandTextResult {
    pub fn from_str(str: &str) -> Self {
        SlashCommandTextResult {
            msg: str.to_string(),
            read: true,
            format: true,
            voice_type: None,
            generator_type: None,
        }
    }
    pub fn from_str_and_flags(str: &str, read: bool, format: bool) -> Self {
        SlashCommandTextResult {
            msg: str.to_string(),
            read,
            format,
            voice_type: None,
            generator_type: None,
        }
    }
}

pub fn get_argument(command: &Command, index: usize) -> Result<&ArgumentValue> {
    command
        .data
        .options
        .get(index)
        .ok_or(anyhow!("index out of range"))?
        .resolved
        .as_ref()
        .ok_or(anyhow!("could not parse"))
}
impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
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
            let commands = definition::set_application_commands(&guild_id, &ctx.http).await;
            match commands {
                Ok(commands) => {
                    for c in commands {
                        tracing::info!("{:?}", c);
                    }
                }
                Err(e) => {
                    tracing::info!("{}", e.to_string())
                }
            }
        }
        tracing::info!("{} is connected!", ready.user.name);
    }
    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let bot_id = &ctx.cache.current_user_id().await;
        let _ = async move {
            let bot_channel_id = guild_id?
                .to_guild_cached(&ctx.cache)
                .await?
                .voice_states
                .get(&bot_id)?
                .channel_id?;
            let channel_id = guild_id?
                .to_guild_cached(&ctx.cache)
                .await?
                .voice_states
                .get(bot_id)?
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
                .filter(|member| member.user.id.0 != bot_id.0)
                .count();

            if members_count == 0 {
                meta::leave(&ctx, guild_id?).await.ok();
                return Some(());
            }

            let user_id = new.user_id;

            if bot_id.0 == user_id.0 {
                return Some(());
            }

            let user_name = &new
                .member
                .as_ref()?
                .nick
                .as_ref()
                .unwrap_or(&new.member.as_ref()?.user.name);

            // info!(
            //     "old = {:?}\nnew = {:?}\nbot_channel_id = {}\nbot_id = {}\nuser_id = {}",
            //     &old, &new, bot_channel_id, bot_id, user_id
            // );
            let greeting_type = if let Some(ref old) = old {
                if old.channel_id == Some(bot_channel_id) && new.channel_id != old.channel_id {
                    1
                } else if old.channel_id != Some(bot_channel_id) && new.channel_id == old.channel_id
                {
                    0
                } else {
                    return Some(());
                }
            } else {
                0
            };
            //info!("greeting_type = {}", greeting_type);
            let uid = user_id.0 as i64;

            let user_config = self.database.get_user_config_or_default(uid).await;
            let nickname = user_config.read_nickname.unwrap_or(user_name.to_string());
            let greet_text = match greeting_type {
                0 => user_config.hello,
                1 => user_config.bye,
                _ => unreachable!(),
            };
            let text = format!("{}さん、{}", nickname, greet_text)
                .make_read_text(&self.database)
                .await;
            let voice_type = user_config.voice_type.try_into().unwrap();
            play_raw_voice(
                &ctx,
                &text,
                voice_type,
                user_config.generator_type.try_into().unwrap(),
                guild_id?,
            )
            .await;

            Some(())
        }
        .await;
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let bot_id = ctx.cache.current_user_id().await;
        let voice_channel_id = guild
            .voice_states
            .get(&bot_id)
            .and_then(|voice_states| voice_states.channel_id);
        let text_channel_id = msg.channel_id;
        let read_channel_id = self.read_channel_id.lock().await.clone();
        info!("msg = {:?}", &msg);
        if read_channel_id == Some(text_channel_id) {
            if let Some(voice_channel_id) = voice_channel_id {
                if msg.author.id != bot_id {
                    play_voice(&ctx, msg, self).await;
                };
            }
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                // respond instantly with text
                "add" | "rem" | "hello" | "bye" | "join" | "leave" | "mute" | "unmute"
                | "rand_member" | "set_nickname" => {
                    let content =
                        interaction_create_with_text(&self, &command, &ctx, &command.data.name)
                            .await;
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content(match content.as_ref() {
                                        Ok(content) => content.msg.clone(),
                                        Err(error) => error.to_string(),
                                    })
                                })
                        })
                        .await
                    {
                        info!("Cannot respond to slash command: {}", why);
                    } else if let Ok(content) = content {
                        if content.read {
                            let msg = if content.format {
                                content.msg.make_read_text(&self.database).await
                            } else {
                                content.msg
                            };
                            let user_id = command.user.id.0;
                            let user_config = self
                                .database
                                .get_user_config_or_default(user_id as i64)
                                .await;
                            let voice_type =
                                content.voice_type.unwrap_or(user_config.voice_type as u8);
                            let generator_type = content
                                .generator_type
                                .unwrap_or(user_config.generator_type as u8);
                            play_raw_voice(
                                &ctx,
                                &msg,
                                voice_type,
                                generator_type,
                                command.guild_id.unwrap(),
                            )
                            .await;
                        }
                    }
                }
                "info" => {
                    let user_id = command.user.id.0 as i64;
                    let user_config = self.database.get_user_config_or_default(user_id).await;

                    let voice_name = self.database.speaker_id_to_name((user_config.generator_type as u8).try_into().unwrap(), user_config.voice_type as u8).await.unwrap();
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|msg| {
                                    msg.create_embed(|emb| {
                                        emb.fields([(
                                            "nickname",
                                            user_config
                                                .read_nickname
                                                .as_ref()
                                                .unwrap_or(&get_display_name(&command)),
                                            true,
                                        ),("voice",&voice_name,true),("hello",&user_config.hello,true),("bye",&user_config.bye,true)])
                                    })
                                })
                        })
                        .await
                        .ok();
                }
                "set_voice_type" => {
                    
                    
                    let speakers = sqlx::query_as!(VoiceType, "SELECT * FROM speakers")
                        .fetch_all(&self.database)
                        .await
                        .unwrap();
                    let generators = ["COEIROINK", "VOICEVOX"];
                    let menus = generators.iter().map(|gen| {
                        CreateSelectMenu::default()
                            .options(|os| {
                                for speaker in speakers
                                    .iter()
                                    .filter(|x| x.generator_type == gen.to_string())
                                {
                                    os.create_option(|o| {
                                        o.label(format!("{} {}", speaker.name, speaker.style_name))
                                            .value(speaker.id)
                                    });
                                }
                                os
                            }).custom_id(gen)
                            .clone()
                    });
                    let _ = command.create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| {
                                msg.components(|c| {
                                    for menu in menus {
                                        c.create_action_row(|row| {
                                            row.add_select_menu(menu)
                                        });
                                    }
                                    c
                                })
                            })
                    }).await;
                    return;
                }
                "walpha" => {
                    let input = get_argument(&command, 0).unwrap();
                    if let ArgumentValue::String(input) = input {
                        let res = command
                            .create_interaction_response(&ctx.http, |res| {
                                res.kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|msg| {
                                        msg.content(format!("{} を計算するよ！", &input))
                                    })
                            })
                            .await
                            .ok();
                        if let Ok(file_path) = util::simple_wolfram_alpha(input).await {
                            let _ = command
                                .channel_id
                                .send_files(&ctx.http, vec![file_path.as_str()], |m| m.content(""))
                                .await;
                        };
                    }
                }
                _ => (),
            };
        } else if let Interaction::MessageComponent(msg) = interaction {
            if let ComponentType::SelectMenu = msg.data.component_type {
                info!("{:?}", msg.data.values);
                let id:i64 = msg.data.values[0].parse().unwrap();
                let q = query!("SELECT generator_type,style_id FROM speakers WHERE id = ?",id).fetch_one(&self.database).await.unwrap();
                let generator_type = q.generator_type;
                let style_id = q.style_id;
                let user_id = msg.user.id.0;
                let mut user_config = self
                    .database
                    .get_user_config_or_default(user_id as i64)
                    .await;
                user_config.generator_type = Generators::try_from(generator_type.as_str()).unwrap() as i64;
                user_config.voice_type = style_id;
                self.database.update_user_config(&user_config).await;
                let res = msg
                    .create_interaction_response(&ctx.http, |res| {
                        res.kind(InteractionResponseType::UpdateMessage)
                    })
                    .await;
                info!("{:?}", res);
            }
        }
    }
}
