use rand::{thread_rng, Rng};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        interactions::{
            application_command::{self, ApplicationCommandInteraction},
            Interaction, InteractionResponseType,
        },
        prelude::{Ready, VoiceState},
    },
};
use std::{
    collections::HashSet,
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{Seek, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::{
    commands::{meta, definition},
    lib::{
        db::{DictDB, UserConfigDB},
        text::TextMessage,
        voice::{play_raw_voice, play_voice},
    },
    Dict, SlashCommandResult,
};
pub const GUILD_IDS_PATH: &str = "guilds.json";

pub struct Handler {
    pub database: sqlx::SqlitePool,
    pub read_channel_id: Arc<Mutex<Option<serenity::model::id::ChannelId>>>,
}
impl Handler {
    pub async fn hello(
        &self,
        command: &ApplicationCommandInteraction,
        greet: &str,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
        let mut user_config = self.database.get_user_config_or_default(user_id).await;
        user_config.hello = greet.to_string();
        self.database.update_user_config(&user_config).await;
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
        let mut user_config = self.database.get_user_config_or_default(user_id).await;
        user_config.bye = greet.to_string();
        self.database.update_user_config(&user_config).await;
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
        let mut user_config = self.database.get_user_config_or_default(user_id).await;
        user_config.voice_type = voice_type;
        self.database.update_user_config(&user_config).await;
        Ok(format!("ボイスタイプを {} に変えたよ", voice_type).to_string())
    }

    pub async fn set_generator_type(
        &self,
        command: &ApplicationCommandInteraction,
        generator_type: i64,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
        let mut user_config = self.database.get_user_config_or_default(user_id).await;
        user_config.generator_type = generator_type;
        self.database.update_user_config(&user_config).await;
        Ok(format!(
            "{}に変えたよ",
            match generator_type {
                0 => "COEIROINK",
                1 => "VOICEVOX",
                _ => unreachable!(),
            }
        )
        .to_string())
    }
    pub async fn add(&self, before: &str, after: &str) -> SlashCommandResult {
        let dict = Dict {
            word: before.to_string(),
            read_word: after.to_string(),
        };
        self.database.update_dict(&dict).await;
        Ok(format!("これからは、{} を {} って読むね", before, after))
    }
    pub async fn rem(&self, word: &str) -> SlashCommandResult {
        if let Ok(_) = sqlx::query!("DELETE FROM dict WHERE word = ?", word)
            .execute(&self.database)
            .await
        {
            Ok(format!("これからは {} って読むね", word))
        } else {
            Err("その単語は登録されてないよ！".to_string())
        }
    }
    pub async fn set_nickname(
        &self,
        command: &ApplicationCommandInteraction,
        nickname: &str,
    ) -> SlashCommandResult {
        let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
        let mut user_config = self.database.get_user_config_or_default(user_id).await;
        user_config.read_nickname = Some(nickname.to_string());
        tracing::info!("{:?}", user_config);
        self.database.update_user_config(&user_config).await;
        Ok(format!(
            "{}さん、これからは{}って呼ぶね",
            command.member.as_ref().unwrap().user.name,
            nickname.to_string()
        )
        .to_string())
    }
    pub async fn rand_member(&self,command: &ApplicationCommandInteraction,ctx:&Context) -> SlashCommandResult {
        let guild_id = command.guild_id.ok_or("guild does not exist")?;
        let guild = ctx.cache.guild(guild_id).await.ok_or("guild does not exist")?;
        let vc_members = guild.voice_states.keys().collect::<Vec<_>>();
        let len = vc_members.len();
        let mut rng = thread_rng();
        let i = rng.gen_range(0..len);        
        let user_id = *vc_members[i];
        
        let member = ctx.cache.as_ref().member(&guild_id, &user_id).await.ok_or("member not found")?;
        //Ok(format!("でけでけでけでけ・・・でん！{}",member.nick.unwrap_or(member.user.name)))
        //Ok(user_id.to_string())
        unimplemented!()
    }
}

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
            let nako_channel_id = guild_id?
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

            let q = sqlx::query!(
                "SELECT hello,bye,voice_type,generator_type FROM user_config WHERE user_id = ?",
                uid
            )
            .fetch_one(&self.database)
            .await;
            let nickname = self
                    .database
                    .get_user_config_or_default(uid)
                    .await
                    .read_nickname
                    .unwrap_or(
                        user_name.to_string(),
                    );
            if let Ok(q) = q {
                let greet_text = match greeting_type {
                    0 => q.hello,
                    1 => q.bye,
                    _ => unreachable!(),
                };
                
                let text = format!("{}さん、{}", nickname, greet_text)
                    .make_read_text(&self.database)
                    .await;
                let voice_type = q.voice_type.try_into().unwrap();
                play_raw_voice(
                    &ctx,
                    &text,
                    voice_type,
                    q.generator_type.try_into().unwrap(),
                    guild_id?,
                )
                .await;
            } else {
                let greet_text = match greeting_type {
                    0 => "こんにちは",
                    1 => "ばいばい",
                    _ => unreachable!(),
                };

                let text = format!("{}さん、{}", nickname, greet_text)
                    .make_read_text(&self.database)
                    .await;
                play_raw_voice(&ctx, &text, 1, 1, guild_id?).await;
            }
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
                if members.contains(&nako_id) && msg.author.id != nako_id {
                    dbg!(&msg);
                    play_voice(&ctx, msg, self).await;
                };
            }
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let mut voice_type = 1;
            let mut generator_type = 0;
            
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
                        Ok(format!("タイプ{}はこんな感じだよ",voice_type))

                    } else {
                        unreachable!()
                    }
                }
                "set_voice_type" => {
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
                     = voice_type_args {
                        voice_type = (*voice_type_args).try_into().unwrap();
                        self.set_voice_type(&command, *voice_type_args).await
                     } else {
                        unreachable!()
                     }
                }
                "set_generator_type" => {
                    let generator_type_args = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected integer")
                        .resolved
                        .as_ref()
                        .expect("Expected integer");

                    if let application_command::ApplicationCommandInteractionDataOptionValue::Integer(
                            generator_type_args,
                        )
                     = generator_type_args {
                        generator_type = (*generator_type_args).try_into().unwrap();
                        self.set_generator_type(&command, *generator_type_args).await
                     } else {
                        unreachable!()
                     }
                }
                "set_nickname" => {
                    let nickname = &command
                        .data
                        .options
                        .get(0)
                        .expect("Expected string")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    if let application_command::ApplicationCommandInteractionDataOptionValue::String(nickname) = nickname {
                        self.set_nickname(&command,&nickname).await
                    } else {
                        unreachable!()
                    }
                },
                "rand_member" => {
                    //self.rand_member(&command,&ctx).await
                    unimplemented!()
                },
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
                    play_raw_voice(
                        &ctx,
                        &content,
                        voice_type,
                        generator_type,
                        command.guild_id.unwrap(),
                    )
                    .await;
                }
            }
        }
    }
}
