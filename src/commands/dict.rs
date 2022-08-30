type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
use anyhow::{anyhow, Result};

use crate::{lib::db::DictDB, Data, Dict};

#[poise::command(slash_command, description_localized("ja", "辞書に単語を登録します"))]
pub async fn add(
    ctx: Context<'_>,
    #[description = "before"] before: String,
    #[description = "after"] after: String,
) -> Result<()> {
    let dict = Dict {
        word: before.to_string(),
        read_word: after.to_string(),
    };
    ctx.data().database.update_dict(&dict).await?;
    ctx.say(format!("これからは {} を {} って読むね", before, after))
        .await?;
    Ok(())
}
#[poise::command(slash_command, description_localized("ja", "辞書から単語を削除します"))]
pub async fn rem(ctx: Context<'_>, #[description = "word"] word: String) -> Result<()> {
    if (ctx.data().database.remove(&word).await).is_ok() {
        ctx.say(format!("これからは {} って読むね", word)).await?;
        Ok(())
    } else {
        Err(anyhow!("その単語は登録されてないよ！"))
    }
}
