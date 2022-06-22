use regex;
use serenity::client::Context;

use crate::Handler;

pub const DICT_PATH: &str = "read_dict.json";
pub const GREETING_DICT_PATH: &str = "greeting_dict.json";
pub const VOICE_TYPE_DICT_PATH: &str = "voice_type_dict.json";
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
}
impl Text {
    pub fn new(text: String) -> Self {
        Text { text }
    }
    fn replace_url(&self) -> Self {
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='\[\]]+").unwrap();
        Self::new(re.replace_all(&self.text, "URL").to_string())
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        Self::new(re.replace_all(&self.text, "").to_string())
    }
    async fn replace_by_dict(&self, handler: &Handler) -> Self {
        let mut text = self.text.clone();
        let dict = sqlx::query!("SELECT word,read_word FROM dict")
            .fetch_all(&handler.dict)
            .await
            .unwrap();
        for w in dict.iter() {
            let before = &w.word;
            let after = &w.read_word;
            text = text.replace(before, after);
        }
        Self::new(text)
    }
    fn remove_custom_emoji(&self) -> Self {
        let re = regex::Regex::new(r"<@.+?>").unwrap();
        Self::new(re.replace_all(&self.text, "").to_string())
    }
    pub async fn make_read_text(&self, handler:&Handler) -> Self {
        self.replace_url()
            .remove_spoiler()
            .remove_custom_emoji()
            .replace_by_dict(handler)
            .await
    }
}
