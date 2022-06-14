use std::{collections::HashMap};

use regex;

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
    fn replace_by_dict(&self, dict: &HashMap<String, String>) -> Self {
        let mut text = self.text.clone();
        for (k, v) in dict.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
    }
    pub fn make_read_text(&self, dict: &HashMap<String, String>) -> Self {
        self.replace_url().remove_spoiler().replace_by_dict(dict)
    }
}
