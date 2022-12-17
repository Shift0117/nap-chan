use crate::{lib::db::UserConfigDB, Context};
use anyhow::{anyhow, Result};
use poise::serenity_prelude::{CreateSelectMenu, InteractionResponseType};

pub async fn get_display_name(ctx: Context<'_>) -> Result<String> {
    Ok(ctx
        .author_member()
        .await
        .ok_or_else(|| anyhow!("member not found"))?
        .nick
        .as_ref()
        .unwrap_or(&ctx.author_member().await.unwrap().user.name)
        .to_string())
}

#[poise::command(
    slash_command,
    description_localized("ja", "ボイスタイプ設定のためのメニューを表示します")
)]
pub async fn set_voice_type(ctx: Context<'_>) -> Result<()> {
    let mut menus = Vec::new();
    let voice_types = ctx.data().voice_types.lock().await;
    for (idx, vec) in voice_types.chunks(25).enumerate() {
        let menu = CreateSelectMenu::default()
            .options(|os| {
                for (speaker_idx, speaker) in vec.iter().enumerate() {
                    os.create_option(|op| {
                        op.label(format!("{} {}", speaker.name, speaker.style_name))
                            .value(speaker_idx + 25 * idx)
                    });
                }
                os
            })
            .custom_id(idx)
            .clone();
        menus.push(menu);
    }
    if let poise::Context::Application(ctx) = ctx {
        if let poise::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(cmd) =
            ctx.interaction
        {
            cmd.create_interaction_response(ctx.discord, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|msg| {
                        msg.components(|c| {
                            for menu in menus {
                                c.create_action_row(|row| row.add_select_menu(menu));
                            }
                            c
                        })
                    })
            })
            .await?;
        }
    }
    Ok(())
}
#[poise::command(
    slash_command,
    description_localized("ja", "入室時のあいさつを設定します。")
)]
pub async fn set_hello(ctx: Context<'_>, #[description = "greet"] greet: String) -> Result<()> {
    let user_id = ctx.author().id.0 as i64;
    let mut user_config = ctx
        .data()
        .database
        .get_user_config_or_default(user_id)
        .await?;

    user_config.hello = greet.to_string();
    let name = get_display_name(ctx).await?;

    ctx.data().database.update_user_config(&user_config).await?;
    ctx.say(format!(
        "{} さん、これからは {} ってあいさつするね",
        name, greet
    ))
    .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("ja", "退室時のあいさつを設定します。")
)]
pub async fn set_bye(ctx: Context<'_>, #[description = "greet"] greet: String) -> Result<()> {
    let user_id = ctx.author().id.0 as i64;
    let mut user_config = ctx
        .data()
        .database
        .get_user_config_or_default(user_id)
        .await?;

    user_config.bye = greet.to_string();
    let name = get_display_name(ctx).await?;

    ctx.data().database.update_user_config(&user_config).await?;
    ctx.say(format!(
        "{} さん、これからは {} ってあいさつするね",
        name, greet
    ))
    .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized(
        "ja",
        "読み上げる名前を設定します。引数を与えなかった場合、名前を読まないようにします"
    )
)]
pub async fn set_nickname(
    ctx: Context<'_>,
    #[description = "greet"] nickname: Option<String>,
) -> Result<()> {
    let user_id = ctx.author().id.0 as i64;
    let mut user_config = ctx
        .data()
        .database
        .get_user_config_or_default(user_id)
        .await?;
    let author_name = get_display_name(ctx).await?;
    let say_text = match &nickname {
        Some(nickname) => format!("{}さん、これからは{}って呼ぶね", author_name, nickname),
        None => format!("{}さん、これからは名前を呼ばないよ", author_name),
    };
    let nickname = nickname.unwrap_or_default();
    user_config.read_nickname = Some(nickname.to_string());
    tracing::info!("{:?}", user_config);
    ctx.data().database.update_user_config(&user_config).await?;
    ctx.say(say_text).await?;
    Ok(())
}
