use regex;
use serenity::client::Context;

use crate::commands::dict::DictHandler;
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
    async fn replace_by_dict(&self, ctx: &Context) -> Self {
        let mut text = self.text.clone();
        let dicts_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<DictHandler>().unwrap().clone()
        };
        let dicts = dicts_lock.lock().await;
        for (k, v) in dicts.dict.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
    }
    fn remove_custom_emoji(&self) -> Self {
        let re = regex::Regex::new(r"<@.+?>").unwrap();
        Self::new(re.replace_all(&self.text, "").to_string())
    }
    pub async fn make_read_text(&self, ctx: &Context) -> Self {
        self.replace_url()
            .remove_spoiler()
            .remove_custom_emoji()
            .replace_by_dict(ctx)
            .await
    }
}
