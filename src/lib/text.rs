use std::{
    io::{Read, Stdout},
    process::{Command, Stdio},
};

use regex;
use serenity::async_trait;

use super::db::DictDB;
use wana_kana::{
    is_katakana::is_katakana,
    is_romaji::is_romaji,
    to_kana::*,
    to_katakana::{to_katakana, to_katakana_with_opt},
};
#[async_trait]
pub trait TextMessage {
    fn replace_url(&self) -> Self;
    fn remove_spoiler(&self) -> Self;
    async fn replace_by_dict(&self, database: &sqlx::SqlitePool) -> Self;
    fn remove_custom_emoji(&self) -> Self;
    async fn make_read_text(&self, database: &sqlx::SqlitePool) -> Self;
    fn hiraganize(&self) -> Self;
}
#[async_trait]
impl TextMessage for String {
    fn replace_url(&self) -> Self {
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='\[\]]+").unwrap();
        re.replace_all(self, "URL").to_string()
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        re.replace_all(self, "").to_string()
    }
    async fn replace_by_dict(&self, database: &sqlx::SqlitePool) -> Self {
        let mut text = self.clone();
        for w in database.get_dict_all().await {
            tracing::info!("{:?} {:?}", &w, &text);
            let before = &w.word;
            let after = &w.read_word;
            text = text.replace(before, after);
        }
        text
    }
    fn hiraganize(&self) -> Self {
        let re = regex::Regex::new(r"[a-zA-Z]+").unwrap();
        let mut text = self.clone();
        for c in re.captures_iter(self) {
            if let Some(english_match) = c.get(0) {
                let english = english_match.as_str();
                let res = Command::new("alkana")
                    .arg(english)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("alkana command failed to start");

                let mut out = res.stdout.unwrap();
                let mut buf = String::new();
                out.read_to_string(&mut buf).ok();
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                if !buf.is_empty() {
                    text = text.replace(&english, &buf);
                } else {
                    let katakana = to_katakana(english);
                    if is_katakana(&katakana) {
                        text = text.replace(&english, &katakana);
                    }
                }
            }
        }
        text
    }
    fn remove_custom_emoji(&self) -> Self {
        let re = regex::Regex::new(r"<@.+?>").unwrap();
        re.replace_all(self, "").to_string()
    }
    async fn make_read_text(&self, database: &sqlx::SqlitePool) -> Self {
        self.replace_url()
            .remove_spoiler()
            .remove_custom_emoji()
            .replace_by_dict(database)
            .await
            .hiraganize()
    }
}

#[test]
fn hiraganize_test() {
    let word = "hello".to_string();
    assert_eq!("ハロー".to_string(), word.hiraganize());

    let sentence = "hello world".to_string();
    assert_eq!("ハロー ワールド".to_string(), sentence.hiraganize());

    let hiragana = "はろーわーるど".to_string();
    assert_eq!("はろーわーるど".to_string(), hiragana.hiraganize());

    let mixed = "hello てすと world".to_string();
    assert_eq!("ハロー てすと ワールド".to_string(), mixed.hiraganize());

    let romaji = "honyaraka".to_string();
    assert_eq!("ホニャラカ", romaji.hiraganize());

    let unknown = "sfhsakhba".to_string();
    assert_eq!(unknown, unknown.hiraganize());
}
