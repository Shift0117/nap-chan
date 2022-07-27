# nap-chan
[nakochan](https://github.com/niuez/nakochan) を Rust で書くことを目的として作られた、 discord の読み上げ bot です。
VOICEVOX api と同じ形式の任意のアプリに対応しています。(e.g. COEIROINK)
複数のアプリの同時起動にも対応しています。

# 導入方法
[install.md](./install.md)を参照してください。


# 機能
- 読み上げ
  - spoiler,code block 内の文章は読まない
  - 英語に一部対応
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
