// use anyhow::{anyhow, Result};

// use serenity::{
//     async_trait,
//     builder::CreateSelectMenu,
//     client::{Context, EventHandler},
//     model::{
//         application::{
//             component::ComponentType,
//             interaction::application_command::{
//                 ApplicationCommandInteraction, CommandDataOptionValue,
//             },
//             interaction::{Interaction, InteractionResponseType},
//         },
//         channel::Message,
//         prelude::{Ready, VoiceState},
//     },
//     utils::ContentSafeOptions,
// };
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tracing::info;

// use crate::{
//     commands::{
//         definition,
//         interactions::{get_display_name, interaction_create_with_text},
//         meta, util,
//     },
//     lib::{
//         db::{UserConfigDB, VoiceType},
//         text::TextMessage,
//         voice::VoiceOptions,
//     },
// };

// pub struct Handler {
//     pub database: sqlx::SqlitePool,
//     pub read_channel_id: Arc<Mutex<Option<serenity::model::id::ChannelId>>>,
//     pub voice_types: Arc<Mutex<Vec<VoiceType>>>,
// }
// pub type Command = ApplicationCommandInteraction;
// pub type ArgumentValue = CommandDataOptionValue;
// #[derive(Clone)]
// pub struct SlashCommandTextResult {
//     msg: String,
//     read: bool,
//     format: bool,
//     voice_type: Option<u32>,
//     generator_type: Option<usize>,
// }

// impl SlashCommandTextResult {
//     pub fn from_str(str: &str) -> Self {
//         SlashCommandTextResult {
//             msg: str.to_string(),
//             read: true,
//             format: true,
//             voice_type: None,
//             generator_type: None,
//         }
//     }
//     pub fn from_str_and_flags(str: &str, read: bool, format: bool) -> Self {
//         SlashCommandTextResult {
//             msg: str.to_string(),
//             read,
//             format,
//             voice_type: None,
//             generator_type: None,
//         }
//     }
// }

// pub fn get_argument(command: &Command, index: usize) -> Result<&ArgumentValue> {
//     command
//         .data
//         .options
//         .get(index)
//         .ok_or_else(|| anyhow!("index out of range"))?
//         .resolved
//         .as_ref()
//         .ok_or_else(|| anyhow!("could not parse"))
// }
// impl Handler {}

// #[async_trait]
// impl EventHandler for Handler {
//     async fn ready(&self, ctx: Context, ready: Ready) {
//         let commands = definition::set_application_commands(&ctx.http).await;
//         match commands {
//             Ok(commands) => {
//                 for c in commands {
//                     tracing::info!("{:?}", c);
//                 }
//             }
//             Err(e) => {
//                 tracing::info!("{}", e.to_string())
//             }
//         }
//         /*let old_global_commands = ctx.http.get_global_application_commands().await.unwrap();
//         for command in old_global_commands {
//             dbg!(command.name);
//             ctx.http.delete_global_application_command(command.id.0).await;
//         }*/

//         tracing::info!("{} is connected!", ready.user.name);
//     }
//     async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
//         let bot_id = &ctx.cache.current_user_id();
//         let guild_id = new.guild_id;
//         let _ = async move {
//             let bot_channel_id = guild_id?
//                 .to_guild_cached(&ctx.cache)?
//                 .voice_states
//                 .get(bot_id)?
//                 .channel_id?;

//             let members_count = ctx
//                 .cache
//                 .channel(bot_channel_id)?
//                 .guild()?
//                 .members(&ctx.cache)
//                 .await
//                 .ok()?
//                 .iter()
//                 .filter(|member| member.user.id.0 != bot_id.0)
//                 .count();

//             if members_count == 0 {
//                 meta::leave(&ctx, guild_id?).await.ok();
//                 return Some(());
//             }

//             let user_id = new.user_id;

//             if bot_id.0 == user_id.0 {
//                 return Some(());
//             }

//             let user_name = &new
//                 .member
//                 .as_ref()?
//                 .nick
//                 .as_ref()
//                 .unwrap_or(&new.member.as_ref()?.user.name);

//             info!(
//                 "old = {:?}\nnew = {:?}\nbot_channel_id = {}\nbot_id = {}\nuser_id = {}",
//                 &old, &new, bot_channel_id, bot_id, user_id
//             );

//             // bye iff old.is_some and (new.channel neq old.channel) and (old.channel = bot.channel)
//             // hello iff (new.channel = bot.channel) and (old.is_none or old.channel != bot.channel)

//             let greeting_type = if old.is_some()
//                 && new.channel_id != old.as_ref().unwrap().channel_id
//                 && old.as_ref().unwrap().channel_id == Some(bot_channel_id)
//             {
//                 1
//             } else if new.channel_id == Some(bot_channel_id)
//                 && (old.is_none() || old.unwrap().channel_id != Some(bot_channel_id))
//             {
//                 0
//             } else {
//                 return Some(());
//             };

//             let uid = user_id.0 as i64;
//             let user_config = self.database.get_user_config_or_default(uid).await.unwrap();
//             let nickname = user_config
//                 .read_nickname
//                 .unwrap_or_else(|| user_name.to_string());
//             let greet_text = match greeting_type {
//                 0 => user_config.hello,
//                 1 => user_config.bye,
//                 _ => unreachable!(),
//             };
//             let text = format!("{}さん、{}", nickname, greet_text)
//                 .make_read_text(&self.database)
//                 .await;
//             let voice_type = user_config.voice_type;
//             let generator_type = user_config.generator_type;
//             if let Err(e) = VoiceOptions::new()
//                 .voice_type(voice_type)
//                 .generator_type(generator_type)
//                 .play_voice(&ctx, guild_id?, text)
//                 .await
//             {
//                 info!("{}", e);
//             };
//             Some(())
//         }
//         .await;
//     }
//     async fn message(&self, ctx: Context, msg: Message) {
//         info!("{:?}", &msg);
//         let user_config = self
//             .database
//             .get_user_config_or_default(msg.author.id.0 as i64)
//             .await
//             .unwrap();
//         let voice_type = user_config.voice_type;
//         let generator_type = user_config.generator_type;
//         let nickname = user_config.read_nickname.unwrap_or_else(|| {
//             msg.member
//                 .as_ref()
//                 .unwrap()
//                 .nick
//                 .as_ref()
//                 .unwrap_or(&msg.author.name)
//                 .to_string()
//         });
//         info!("{:?}", &nickname);
//         let guild = msg.guild(&ctx.cache).unwrap();
//         let bot_id = ctx.cache.current_user_id();
//         let voice_channel_id = guild
//             .voice_states
//             .get(&bot_id)
//             .and_then(|voice_states| voice_states.channel_id);
//         let text_channel_id = msg.channel_id;
//         let read_channel_id = *self.read_channel_id.lock().await;
//         if read_channel_id == Some(text_channel_id) {
//             if let Some(_voice_channel_id) = voice_channel_id {
//                 if msg.author.id != bot_id {
//                     if let Err(e) = VoiceOptions::new()
//                         .clean(Some(&ContentSafeOptions::new()))
//                         .dict(Some(&self.database))
//                         .read_name(Some(&nickname))
//                         .generator_type(generator_type)
//                         .voice_type(voice_type)
//                         .play_voice(&ctx, guild.id, msg.content)
//                         .await
//                     {
//                         info!("error: {}", e)
//                     };
//                 };
//             }
//         }
//     }

//     async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
//         if let Interaction::ApplicationCommand(command) = interaction {
//             match command.data.name.as_str() {
//                 // respond instantly with text
//                 "add" | "rem" | "hello" | "bye" | "join" | "leave" | "mute" | "unmute"
//                 | "rand_member" | "set_nickname" => {
//                     let content =
//                         interaction_create_with_text(self, &command, &ctx, &command.data.name)
//                             .await;
//                     if let Err(why) = command
//                         .create_interaction_response(&ctx.http, |response| {
//                             response
//                                 .kind(InteractionResponseType::ChannelMessageWithSource)
//                                 .interaction_response_data(|message| {
//                                     message.content(match content.as_ref() {
//                                         Ok(content) => content.msg.clone(),
//                                         Err(error) => error.to_string(),
//                                     })
//                                 })
//                         })
//                         .await
//                     {
//                         info!("Cannot respond to slash command: {}", why);
//                     } else if let Ok(content) = content {
//                         if content.read {
//                             let msg = if content.format {
//                                 content.msg.make_read_text(&self.database).await
//                             } else {
//                                 content.msg
//                             };
//                             let user_id = command.user.id.0;
//                             let user_config = self
//                                 .database
//                                 .get_user_config_or_default(user_id as i64)
//                                 .await
//                                 .unwrap();
//                             let voice_type =
//                                 content.voice_type.unwrap_or(user_config.voice_type as u32);
//                             let generator_type = content
//                                 .generator_type
//                                 .unwrap_or(user_config.generator_type as usize);
//                             if let Err(e) = VoiceOptions::new()
//                                 .voice_type(voice_type as i64)
//                                 .generator_type(generator_type as i64)
//                                 .play_voice(&ctx, command.guild_id.unwrap(), msg)
//                                 .await
//                             {
//                                 info!("{}", e);
//                             };
//                         }
//                     }
//                 }
//                 "info" => {
//                     let user_id = command.user.id.0 as i64;
//                     let user_config = self
//                         .database
//                         .get_user_config_or_default(user_id)
//                         .await
//                         .unwrap();
//                     let voice_type: VoiceType = self
//                         .voice_types
//                         .lock()
//                         .await
//                         .iter()
//                         .find(|voice_type| {
//                             voice_type.generator_type == user_config.generator_type
//                                 && voice_type.style_id as i64 == user_config.voice_type
//                         })
//                         .unwrap()
//                         .clone();
//                     command
//                         .create_interaction_response(&ctx.http, |response| {
//                             response
//                                 .kind(InteractionResponseType::ChannelMessageWithSource)
//                                 .interaction_response_data(|msg| {
//                                     msg.embed(|emb| {
//                                         emb.fields([
//                                             (
//                                                 "nickname",
//                                                 user_config
//                                                     .read_nickname
//                                                     .as_ref()
//                                                     .unwrap_or(&get_display_name(&command)),
//                                                 true,
//                                             ),
//                                             (
//                                                 "voice",
//                                                 &format!(
//                                                     "{} {}",
//                                                     voice_type.name, voice_type.style_name
//                                                 ),
//                                                 true,
//                                             ),
//                                             ("hello", &user_config.hello, true),
//                                             ("bye", &user_config.bye, true),
//                                         ])
//                                     })
//                                 })
//                         })
//                         .await
//                         .ok();
//                 }
//                 "set_voice_type" => {
//                     let mut menus = Vec::new();
//                     let voice_types = self.voice_types.lock().await;
//                     for (idx, vec) in voice_types.chunks(25).enumerate() {
//                         let menu = CreateSelectMenu::default()
//                             .options(|os| {
//                                 for (speaker_idx, speaker) in vec.iter().enumerate() {
//                                     os.create_option(|op| {
//                                         op.label(format!("{} {}", speaker.name, speaker.style_name))
//                                             .value(speaker_idx + 25 * idx)
//                                     });
//                                 }
//                                 os
//                             })
//                             .custom_id(idx)
//                             .clone();
//                         menus.push(menu);
//                     }

//                     let e = command
//                         .create_interaction_response(&ctx.http, |response| {
//                             response
//                                 .kind(InteractionResponseType::ChannelMessageWithSource)
//                                 .interaction_response_data(|msg| {
//                                     msg.components(|c| {
//                                         for menu in menus {
//                                             c.create_action_row(|row| row.add_select_menu(menu));
//                                         }
//                                         c
//                                     })
//                                 })
//                         })
//                         .await;
//                     if e.is_err() {
//                         info!("{:?}", e);
//                     }
//                     return;
//                 }
//                 "walpha" => {
//                     let input = get_argument(&command, 0).unwrap();
//                     if let ArgumentValue::String(input) = input {
//                         let _res = command
//                             .create_interaction_response(&ctx.http, |res| {
//                                 res.kind(InteractionResponseType::ChannelMessageWithSource)
//                                     .interaction_response_data(|msg| {
//                                         msg.content(format!("{} を計算するよ！", &input))
//                                     })
//                             })
//                             .await
//                             .ok();
//                         if let Ok(file_path) = util::simple_wolfram_alpha(input).await {
//                             let _ = command
//                                 .channel_id
//                                 .send_files(&ctx.http, vec![file_path.as_str()], |m| m.content(""))
//                                 .await;
//                         };
//                     }
//                 }
//                 "help" => {
//                     util::help(&ctx.http, &command).await.unwrap();
//                 }
//                 _ => (),
//             };
//         } else if let Interaction::MessageComponent(msg) = interaction {
//             if let ComponentType::SelectMenu = msg.data.component_type {
//                 info!("{:?}", msg.data.values);
//                 let idx = msg.data.values[0].parse::<usize>().unwrap();
//                 let voice_type = &self.voice_types.lock().await[idx];
//                 let generator_type = voice_type.generator_type;
//                 let style_id = voice_type.style_id;
//                 let user_id = msg.user.id.0;
//                 let mut user_config = self
//                     .database
//                     .get_user_config_or_default(user_id as i64)
//                     .await
//                     .unwrap();
//                 user_config.generator_type = generator_type;
//                 user_config.voice_type = style_id as i64;
//                 self.database
//                     .update_user_config(&user_config)
//                     .await
//                     .unwrap();
//                 let res = msg
//                     .create_interaction_response(&ctx.http, |res| {
//                         res.kind(InteractionResponseType::UpdateMessage)
//                     })
//                     .await;
//                 info!("{:?}", res);
//             }
//         }
//     }
// }
