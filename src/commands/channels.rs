use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "チャンネル一覧を取得"]
async fn all_channels(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = *msg.guild_id.unwrap().as_u64();
    let channels = ctx.http.get_channels(guild_id).await.unwrap();

    for chan in channels {
        msg.channel_id
            .say(&ctx.http, format!("{}", chan.name))
            .await?;
    }
    Ok(())
}