use regex;
use serenity::async_trait;

use crate::handler;

use super::db::DictDB;

#[async_trait]
pub trait TextMessage {
    fn replace_url(&self) -> Self;
    fn remove_spoiler(&self) -> Self;
    async fn replace_by_dict(&self, database: &sqlx::SqlitePool) -> Self;
    fn remove_custom_emoji(&self) -> Self;
    async fn make_read_text(&self, database: &sqlx::SqlitePool) -> Self;
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
            let before = &w.word;
            let after = &w.read_word;
            text = text.replace(before, after);
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
    }
}
