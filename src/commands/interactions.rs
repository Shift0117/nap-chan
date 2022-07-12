use crate::{
    handler::{get_argument, ArgumentValue, Command, Handler, SlashCommandTextResult},
    lib::db::{DictDB, UserConfigDB},
    Dict,
};
use anyhow::{anyhow, Result};
use serenity::client::Context;

use super::{meta, util};

pub fn get_display_name(command: &Command) -> String {
    command
        .member
        .as_ref()
        .unwrap()
        .nick
        .as_ref()
        .unwrap_or(&command.user.name)
        .to_string()
}

pub async fn interaction_create_with_text(
    handler: &Handler,
    command: &Command,
    ctx: &Context,
    command_name: &str,
) -> Result<SlashCommandTextResult> {
    match command_name {
        "join" => meta::join(&ctx, &command, &handler.read_channel_id)
            .await
            .map(|_| SlashCommandTextResult::from_str("おはよ！")),
        "leave" => meta::leave(&ctx, command.guild_id.unwrap())
            .await
            .map(|_| SlashCommandTextResult::from_str("ばいばい")),
        "add" => {
            let before = get_argument(command, 0)?;
            let after = get_argument(command, 1)?;
            if let (ArgumentValue::String(before), ArgumentValue::String(after)) = (before, after) {
                let dict = Dict {
                    word: before.to_string(),
                    read_word: after.to_string(),
                };
                handler.database.update_dict(&dict).await?;
                Ok(SlashCommandTextResult::from_str_and_flags(
                    &format!("これからは、{} を {} って読むね", before, after),
                    true,
                    false,
                ))
            } else {
                unreachable!()
            }
        }
        "rem" => {
            let word = get_argument(&command, 0)?;
            if let ArgumentValue::String(word) = word {
                if let Ok(_) = handler.database.remove(word).await {
                    Ok(SlashCommandTextResult::from_str(&format!(
                        "これからは {} って読むね",
                        word
                    )))
                } else {
                    Err(anyhow!("その単語は登録されてないよ！"))
                }
            } else {
                unreachable!()
            }
        }
        "mute" => meta::mute(&ctx, &command)
            .await
            .map(|_| SlashCommandTextResult::from_str("ミュートしたよ")),
        "unmute" => meta::unmute(&ctx, &command)
            .await
            .map(|_| SlashCommandTextResult::from_str("ミュート解除したよ")),
        "hello" => {
            let greet = get_argument(&command, 0)?;
            if let ArgumentValue::String(greet) = greet {
                let user_id = command.member.as_ref().unwrap().user.id.0 as i64;

                let mut user_config = handler.database.get_user_config_or_default(user_id).await?;
                user_config.hello = greet.to_string();
                handler.database.update_user_config(&user_config).await?;
                Ok(SlashCommandTextResult::from_str(&format!(
                    "{}さん、これから{}ってあいさつするね",
                    get_display_name(&command),
                    greet
                )))
            } else {
                unreachable!()
            }
        }
        "bye" => {
            let greet = get_argument(command, 0)?;
            if let ArgumentValue::String(greet) = greet {
                let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
                let mut user_config = handler.database.get_user_config_or_default(user_id).await?;
                user_config.bye = greet.to_string();
                handler.database.update_user_config(&user_config).await?;
                Ok(SlashCommandTextResult::from_str(&format!(
                    "{}さん、これから{}ってあいさつするね",
                    get_display_name(&command),
                    greet
                )))
            } else {
                unreachable!()
            }
        }
        "set_nickname" => {
            let nickname = get_argument(command, 0)?;
            if let ArgumentValue::String(nickname) = nickname {
                let user_id = command.member.as_ref().unwrap().user.id.0 as i64;
                let mut user_config = handler.database.get_user_config_or_default(user_id).await?;
                user_config.read_nickname = Some(nickname.to_string());
                tracing::info!("{:?}", user_config);
                handler.database.update_user_config(&user_config).await?;
                Ok(SlashCommandTextResult::from_str(&format!(
                    "{}さん、これからは{}って呼ぶね",
                    get_display_name(&command),
                    nickname.to_string()
                )))
            } else {
                unreachable!()
            }
        }
        "rand_member" => util::rand_member(&command, &ctx).await.map(|member| {
            SlashCommandTextResult::from_str(&format!(
                "でけでけでけでけ・・・でん！{}",
                &member.nick.unwrap_or(member.user.name)
            ))
        }),
        "walpha" => {
            let input = get_argument(command, 0)?;
            if let ArgumentValue::String(input) = input {
                Ok(SlashCommandTextResult::from_str(&format!(
                    "{} を計算するよ！",
                    input
                )))
            } else {
                unreachable!()
            }
        }

        _ => unreachable!(),
    }
}
