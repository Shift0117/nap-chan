use anyhow::Result;
use serenity::{
    http::Http,
    model::{
        id::GuildId,
        interactions::application_command::{self, ApplicationCommand},
    },
};
pub async fn set_application_commands(
    guild_id: &GuildId,
    http: &Http,
) -> Result<Vec<ApplicationCommand>> {
    GuildId::set_application_commands(&guild_id, http, |commands| {
        commands
            .create_application_command(|command| {
                command.name("join").description("VCに参加します")
            })
            .create_application_command(|command| {
                command.name("leave").description("VCから抜けます")
            })
            .create_application_command(|command| {
                command
                    .name("add")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("before")
                            .description("string")
                    })
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .description("string")
                            .name("after")
                    })
                    .description("before を after と読むようにします")
            })
            .create_application_command(|command| {
                command
                    .name("rem")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("word")
                            .description("string")
                    })
                    .description("word の読み方を忘れます")
            })
            .create_application_command(|command| {
                command.name("mute").description("botをミュートします")
            })
            .create_application_command(|command| {
                command
                    .name("unmute")
                    .description("botのミュートを解除します")
            })
            .create_application_command(|command| {
                command
                    .name("hello")
                    .description("入った時のあいさつを変えます")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("greet")
                            .description("string")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("bye")
                    .description("出た時のあいさつを変えます")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("greet")
                            .description("string")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("play_sample_voice")
                    .description("入力されたタイプのサンプルボイスを再生します")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::Integer)
                            .max_int_value(5)
                            .min_int_value(0)
                            .required(true)
                            .name("type")
                            .description("0 から 5 の整数値")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("set_voice_type")
                    .description("ボイスタイプを変えます")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::Integer)
                            .max_int_value(5)
                            .min_int_value(0)
                            .required(true)
                            .name("type")
                            .description("0 から 5 の整数値")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("set_generator_type")
                    .description("音声合成に使うソフトを設定します。0:COEIROINK,1:VOICEVOX")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::Integer)
                            .max_int_value(1)
                            .min_int_value(0)
                            .required(true)
                            .name("type")
                            .description("0 から 1 の整数値")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("set_nickname")
                    .description("呼ぶ名前を設定します。")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("nick")
                            .description("string")
                    })
            })
            .create_application_command(|command| {
                command
                    .name("rand_member")
                    .description("VC内の人をランダムに選びます")
            })
            .create_application_command(|command| {
                command
                    .name("walpha")
                    .description("計算などをしてくれます")
                    .create_option(|option| {
                        option
                            .kind(application_command::ApplicationCommandOptionType::String)
                            .required(true)
                            .name("input")
                            .description("string")
                    })
            })
    })
    .await
    .map_err(|e| e.into())
}
