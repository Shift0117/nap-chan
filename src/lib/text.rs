use std::collections::HashMap;

use regex;
use serenity::client::Context;

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
        Self::new(re.replace(&self.text, "URL").to_string())
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        Self::new(re.replace(&self.text, "").to_string())
    }
    async fn replace_by_dict(&self, ctx: &Context) -> Self {
        let mut text = self.text.clone();
        let dict_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<crate::DictHandler>().unwrap().clone()
        };
        let dict = dict_lock.lock().await;
        for (k, v) in dict.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
    }
    pub async fn make_read_text(&self, ctx: &Context) -> Self {
        self.replace_url().remove_spoiler().replace_by_dict(ctx).await
    }
}
