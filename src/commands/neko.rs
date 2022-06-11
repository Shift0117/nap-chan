use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description = "猫のように鳴く"]
async fn neko(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.channel_id.say で、channel_id の channel にメッセージを投稿
    msg.channel_id
        .say(&ctx.http, format!("{} にゃーん", msg.author.mention()))
        .await?;
    // CommandResultはResultを継承している
    // `Result?` は正常な値の場合、Resultの中身を返し、エラーの場合は即座にreturnする演算子
    Ok(())
}