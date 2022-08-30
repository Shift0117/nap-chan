use std::sync::Arc;

use crate::TrackEndNotifier;
type Context<'a> = poise::Context<'a, crate::Data, anyhow::Error>;
use anyhow::{anyhow, Result};
use serenity::model::{
    application::interaction::application_command::ApplicationCommandInteraction,
    id::{ChannelId, GuildId},
};
use songbird::{Event, TrackEvent};
use tokio::sync::Mutex;

// pub async fn join(
//     ctx: &Context,
//     command: &ApplicationCommandInteraction,
//     read_channel_id: &Arc<Mutex<Option<ChannelId>>>,
// ) -> Result<()> {
//     let guild_id = command
//         .guild_id
//         .ok_or_else(|| anyhow!("guild id not found"))?;
//     let author_id = command
//         .member
//         .as_ref()
//         .ok_or_else(|| anyhow!("member not found"))?
//         .user
//         .id;
//     let text_channel_id = command.channel_id;
//     let channel_id = guild_id
//         .to_guild_cached(&ctx.cache)
//         .ok_or_else(|| anyhow!("guild not found"))?
//         .voice_states
//         .get(&author_id)
//         .ok_or_else(|| anyhow!("author not found"))?
//         .channel_id
//         .ok_or_else(|| anyhow!("channel id not found"))?;
//     let connect_to = channel_id;
//     let manager = songbird::get(ctx)
//         .await
//         .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?
//         .clone();
//     let (handle_lock, _) = manager.join(guild_id, connect_to).await;
//     let mut handle = handle_lock.lock().await;
//     handle.deafen(true).await?;
//     handle.add_global_event(Event::Track(TrackEvent::End), TrackEndNotifier);
//     *read_channel_id.lock().await = Some(text_channel_id);
//     Ok(())
// }
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild id not found"))?;
    let author_id = ctx.author().id;
    let text_channel_id = ctx.channel_id();
    let voice_channel_id = ctx
        .guild()
        .ok_or_else(|| anyhow!("guild not found"))?
        .voice_states
        .get(&author_id)
        .ok_or_else(|| anyhow!("author not in voice channel"))?
        .channel_id
        .ok_or_else(|| anyhow!("channel not found"))?;

    let manager = songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?;
    let (handle_lock, err) = manager.join(guild_id, voice_channel_id).await;
    err?;
    let mut handle = handle_lock.lock().await;
    handle.deafen(true).await?;
    handle.add_global_event(Event::Track(TrackEvent::End), TrackEndNotifier);
    *ctx.data().read_channel_id.lock().await = Some(text_channel_id);
    ctx.say("こんにちは");
    Ok(())
}

// pub async fn leave(ctx: &Context, guild_id: GuildId) -> Result<()> {
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();

//     let has_handler = manager.get(guild_id).is_some();
//     if has_handler {
//         manager.remove(guild_id).await?;
//         Ok(())
//     } else {
//         Err(anyhow!("ボイスチャンネルに入ってないよ"))
//     }
// }

#[poise::command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild id not found"))?;
    let manager = songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?;
    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        manager.remove(guild_id).await?;
        ctx.say("ばいばい");
        Ok(())
    } else {
        Err(anyhow!("ボイスチャンネルに入ってないよ"))
    }
}

// pub async fn mute(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
//     let guild_id = command
//         .guild_id
//         .ok_or_else(|| anyhow!("guild id not found"))?;
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//     let handler_lock = manager
//         .get(guild_id)
//         .ok_or_else(|| anyhow!("ボイスチャンネルに入ってないよ"))?;
//     let mut handler = handler_lock.lock().await;

//     if handler.is_mute() {
//         Err(anyhow!("もうミュートしてるよ"))
//     } else if let Err(e) = handler.mute(true).await {
//         Err(e.into())
//     } else {
//         Ok(())
//     }
// }

#[poise::command(slash_command)]
pub async fn mute(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild id not found"))?;
    let manager = songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?;
    let handler_lock = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("ボイスチャンネルに入ってないよ"))?;
    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        Err(anyhow!("もうミュートしてるよ"))
    } else if let Err(e) = handler.mute(true).await {
        Err(e.into())
    } else {
        ctx.say("ミュートしたよ");
        Ok(())
    }
}

// pub async fn unmute(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
//     let guild_id = command
//         .guild_id
//         .ok_or_else(|| anyhow!("guild id not found"))?;
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//     let handler_lock = manager
//         .get(guild_id)
//         .ok_or_else(|| anyhow!("ボイスチャンネルに入ってないよ"))?;
//     let mut handler = handler_lock.lock().await;
//     if let Err(e) = handler.mute(false).await {
//         Err(e.into())
//     } else {
//         Ok(())
//     }
// }

#[poise::command(slash_command)]
pub async fn unmute(ctx: Context<'_>) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild id not found"))?;
    let manager = songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Songbird Voice client placed in at initialisation."))?;
    let handler_lock = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("ボイスチャンネルに入ってないよ"))?;
    let mut handler = handler_lock.lock().await;
    if let Err(e) = handler.mute(false).await {
        Err(e.into())
    } else {
        ctx.say("ミュート解除したよ");
        Ok(())
    }
}
