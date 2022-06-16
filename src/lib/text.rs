use std::{collections::HashMap, sync::Arc};

use regex;
use serenity::{client::Context, prelude::TypeMapKey};
use tokio::sync::Mutex;

use crate::commands::dict::DictHandler;
pub const DICT_PATH: &str = "read_dict.json";
pub const GREETING_DICT_PATH: &str = "greet_dict.json";

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
        let dicts_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<DictHandler>().unwrap().clone()
        };
        dbg!(&dicts_lock);
        let dicts = dicts_lock.lock().await;
        dbg!(&dicts);
        for (k, v) in dicts.dict.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
    }
    pub async fn make_read_text(&self, ctx: &Context) -> Self {
        self.replace_url()
            .remove_spoiler()
            .replace_by_dict(ctx)
            .await
    }
}
