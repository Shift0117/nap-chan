# discord_bot.rs
nakochan https://github.com/niuez/nakochan のような discord の読み上げ bot を目指しています

## voicevox(coeiroink)を使った読み上げbot

### 準備
-- 注意　現状このままでは動きません

1. COEIROINK の `run.exe` を実行します
2. .env.sample に従って .env に bot の token,application id,COEIROINK が動いているアドレス(デフォルトはおそらく http://127.0.0.1:50031 )を入力します



# 機能
- 読み上げ
- コマンド
  - `/join` コマンドを入力した人が入っているボイスチャンネルに合流します
  - `/leave` 入っているボイスチャンネルから抜けます
  - `/mute` / `/unmute` それぞれ bot をミュート/ミュート解除します
  - `/add before after` before を after と読むようにします
  - `/rem word` /add コマンドで登録した word の読み方をリセットします
  - `/hello greet` コマンドを入力した人が入室したときのあいさつを greet に変更します
  - `/set_voice_type` 読み上げボイスタイプを変更する Select menu を表示します
  - 
