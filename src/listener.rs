use crate::{
    lib::{db::UserConfigDB, text::TextMessage, voice::VoiceOptions},
    Data,
};
use anyhow::{anyhow, Result};
use poise::serenity_prelude::{self as serenity, VoiceState};
use tracing::info;

pub async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework: poise::FrameworkContext<'_, crate::Data, anyhow::Error>,
    user_data: &Data,
) -> Result<(), anyhow::Error> {
    match event {
        poise::Event::Ready { data_about_bot } => ready(ctx, data_about_bot).await,
        poise::Event::Message { new_message } => message(ctx, new_message, user_data).await,
        poise::Event::VoiceStateUpdate { old, new } => {
            voice_state_update(ctx, old, new, user_data).await?
        }
        _ => {}
    }
    Ok(())
}

async fn ready(ctx: &serenity::Context, ready: &serenity::Ready) {
    let old_global_commands = ctx.http.get_global_application_commands().await.unwrap();
    for command in old_global_commands {
        dbg!(command.name);
        ctx.http
            .delete_global_application_command(command.id.0)
            .await
            .unwrap();
    }
    info!("{} is connected!", ready.user.name);
}

async fn message(ctx: &serenity::Context, message: &serenity::Message, user_data: &Data) {
    info!("{:?}", message);
    let user_config = crate::lib::db::UserConfigDB::get_user_config_or_default(
        &user_data.database,
        message.author.id.0 as i64,
    )
    .await
    .unwrap();
    let voice_type = user_config.voice_type;
    let generator_type = user_config.generator_type;
    let nickname = user_config.read_nickname.unwrap_or_else(|| {
        message
            .member
            .as_ref()
            .unwrap()
            .nick
            .as_ref()
            .unwrap_or(&message.author.name)
            .to_string()
    });
    info!("{:?}", &nickname);
    let guild = message.guild(&ctx.cache).unwrap();
    let bot_id = ctx.cache.current_user_id();
    let voice_channel_id = guild
        .voice_states
        .get(&bot_id)
        .and_then(|voice_states| voice_states.channel_id);
    let text_channel_id = message.channel_id;
    let read_channel_id = *user_data.read_channel_id.lock().await;
    if read_channel_id == Some(text_channel_id) {
        if let Some(_voice_channel_id) = voice_channel_id {
            if message.author.id != bot_id {
                if let Err(e) = VoiceOptions::new()
                    .clean(Some(&serenity::ContentSafeOptions::new()))
                    .dict(Some(&user_data.database))
                    .read_name(Some(&nickname))
                    .generator_type(generator_type)
                    .voice_type(voice_type)
                    .play_voice(ctx, guild.id, message.content.clone())
                    .await
                {
                    info!("error: {}", e)
                };
            };
        }
    }
}

async fn voice_state_update(
    ctx: &serenity::Context,
    old: &Option<VoiceState>,
    new: &VoiceState,
    user_data: &Data,
) -> Result<()> {
    let bot_id = &ctx.cache.current_user_id();
    let guild_id = new
        .guild_id
        .ok_or_else(|| anyhow!("new guild id not found"))?;

    let bot_channel_id = guild_id
        .to_guild_cached(&ctx.cache)
        .ok_or_else(|| anyhow!("new guild not found"))?
        .voice_states
        .get(bot_id)
        .ok_or_else(|| anyhow!("bot not in voice channel"))?
        .channel_id
        .ok_or_else(|| anyhow!("channel id not found"))?;

    let members_count = ctx
        .cache
        .channel(bot_channel_id)
        .ok_or_else(|| anyhow!("bot channel not found"))?
        .guild()
        .ok_or_else(|| anyhow!("bot guild not found"))?
        .members(&ctx.cache)
        .await?
        .iter()
        .filter(|member| member.user.id.0 != bot_id.0)
        .count();

    if members_count == 0 {
        let manager = songbird::get(ctx)
            .await
            .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?;
        let has_handler = manager.get(guild_id).is_some();
        if has_handler {
            manager.remove(guild_id).await?;
        }
        return Ok(());
    }

    let user_id = new.user_id;

    if bot_id.0 == user_id.0 {
        return Ok(());
    }
    let new_member = new
        .member
        .as_ref()
        .ok_or_else(|| anyhow!("new member not found"))?;

    let user_name = new_member.nick.as_ref().unwrap_or(&new_member.user.name);

    info!(
        "old = {:?}\nnew = {:?}\nbot_channel_id = {}\nbot_id = {}\nuser_id = {}",
        &old, &new, bot_channel_id, bot_id, user_id
    );

    // bye iff old.is_some and (new.channel neq old.channel) and (old.channel = bot.channel)
    // hello iff (new.channel = bot.channel) and (old.is_none or old.channel != bot.channel)

    let greeting_type = if old.is_some()
        && new.channel_id != old.as_ref().unwrap().channel_id
        && old.as_ref().unwrap().channel_id == Some(bot_channel_id)
    {
        1
    } else if new.channel_id == Some(bot_channel_id)
        && (old.is_none() || old.as_ref().unwrap().channel_id != Some(bot_channel_id))
    {
        0
    } else {
        return Ok(());
    };

    let uid = user_id.0 as i64;
    let user_config = user_data
        .database
        .get_user_config_or_default(uid)
        .await
        .unwrap();
    let nickname = user_config
        .read_nickname
        .unwrap_or_else(|| user_name.to_string());
    let greet_text = match greeting_type {
        0 => user_config.hello,
        1 => user_config.bye,
        _ => unreachable!(),
    };
    let text = format!("{}さん、{}", nickname, greet_text)
        .make_read_text(&user_data.database)
        .await;
    let voice_type = user_config.voice_type;
    let generator_type = user_config.generator_type;
    if let Err(e) = VoiceOptions::new()
        .voice_type(voice_type)
        .generator_type(generator_type)
        .play_voice(ctx, guild_id, text)
        .await
    {
        info!("{}", e);
    };
    Ok(())
}
