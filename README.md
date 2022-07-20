# discord_bot.rs
[nakochan](https://github.com/niuez/nakochan) を Rust で書くことを目的として作られた、 discord の voicevox/coeiroink を使った読み上げ bot です。

# 導入方法
[install.md](./install.md)を参照してください。


# 機能
- 読み上げ
- コマンド
  - `/join` コマンドを入力した人が入っているボイスチャンネルに合流します
  - `/leave` 入っているボイスチャンネルから抜けます
  - `/mute` , `/unmute` それぞれ bot をミュート/ミュート解除します
  - `/add before after` before を after と読むようにします
  - `/rem word` /add コマンドで登録した word の読み方をリセットします
  - `/hello greet` コマンドを入力した人が入室したときのあいさつを greet に変更します
  - `/set_voice_type` 読み上げボイスタイプを変更する Select menu を表示します
  - `/info` 現在のユーザー設定を表示します
  - `/rand_member` VC 内のランダムなメンバーを指定します
  - `/set_nickname` 呼ぶ名前を設定します
  - `/walpha` 計算などをしてくれます
  - `/help` ヘルプを表示します
