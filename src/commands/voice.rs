use serenity::{client::Context, model::id::GuildId};

type SlashCommandResult = Result<String, String>;

pub async fn play_sample_voice(
    ctx: &Context,
    guild_id: GuildId,
    voice_type: u8,
) -> SlashCommandResult {
    let path = format!("sample_voice/sample_{}.wav", voice_type);
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await.unwrap();
        source.metadata.source_url = Some(path);
        handler.enqueue_source(source.into());
    }
    Ok(format!("タイプ{}はこんな感じだよ", voice_type).to_string())
}
