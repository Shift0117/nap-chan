use serenity::{client::Context, model::interactions::application_command::ApplicationCommandInteraction, framework::standard::CommandResult};
use songbird::{TrackEvent, Event};

use crate::TrackEndNotifier;

pub async fn join(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    let guild_id = command.guild_id.unwrap();
    let author_id = command.member.as_ref().unwrap().user.id;
    let channel_id = command
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap()
        .voice_states
        .get(&author_id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();
    let connect_to = channel_id;
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let (handle_lock, _) = manager.join(guild_id, connect_to).await;
    let mut handle = handle_lock.lock().await;
    handle.deafen(true).await.unwrap();
    handle.add_global_event(Event::Track(TrackEvent::End), TrackEndNotifier);
    Ok(())
}

pub async fn leave(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    let guild_id = command.guild_id.unwrap();
    let author_id = command.member.as_ref().unwrap().user.id;
    let channel_id = command
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap()
        .voice_states
        .get(&author_id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await?;
        }
        //channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        //channel_id.say(&ctx.http, "Not in a voice channel").await?;
    }
    Ok(())
}

pub async fn mute(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    let guild_id = command.guild_id.unwrap();
    let author_id = command.member.as_ref().unwrap().user.id;
    let channel_id = command
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap()
        .voice_states
        .get(&author_id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            channel_id.say(&ctx.http, "Not in a voice channel").await?;
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
    channel_id.say(&ctx.http, content).await?;
    Ok(())
}

pub async fn unmute(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    let guild_id = command.guild_id.unwrap();
    let author_id = command.member.as_ref().unwrap().user.id;
    let channel_id = command
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap()
        .voice_states
        .get(&author_id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();
    let manager = songbird::get(&ctx)
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
    channel_id.say(&ctx.http, content).await?;
    Ok(())
}
