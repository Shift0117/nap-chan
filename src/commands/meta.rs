
use crate::TrackEndNotifier;
type Context<'a> = poise::Context<'a, crate::Data, anyhow::Error>;
use anyhow::{anyhow, Result};
use songbird::{Event, TrackEvent};
#[poise::command(slash_command, description_localized("ja", "VCに参加します"))]
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
    ctx.say("こんにちは").await?;
    Ok(())
}

#[poise::command(slash_command, description_localized("ja", "VCから抜けます"))]
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
        ctx.say("ばいばい").await?;
        Ok(())
    } else {
        Err(anyhow!("ボイスチャンネルに入ってないよ"))
    }
}

#[poise::command(slash_command, description_localized("ja", "botをミュートします"))]
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
        ctx.say("ミュートしたよ").await?;
        Ok(())
    }
}

#[poise::command(
    slash_command,
    description_localized("ja", "botのミュートを解除します")
)]
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
        ctx.say("ミュート解除したよ").await?;
        Ok(())
    }
}
