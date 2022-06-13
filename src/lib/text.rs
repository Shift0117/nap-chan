use std::{fs::File, io::BufReader, collections::HashMap};

use regex;
use serde_json;

#[derive(Debug,Clone)]
pub struct Text {
    pub text: String,
}
impl Text {
    pub fn new(text:String) -> Self {
        Text { text }
    }
    fn replace_url(&self)-> Self {
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='\[\]]+").unwrap();
        Self::new(re.replace(&self.text, "URL").to_string())
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        Self::new(re.replace(&self.text, "").to_string())
    }
    fn replace_by_dict(&self) -> Self {
        let mut text = self.text.clone();
        let dict_file = File::open("read_dict.json").unwrap();
        let reader = BufReader::new(dict_file);
        let map:Vec<(String,String)> = serde_json::from_reader(reader).unwrap();
        for (k,v) in map.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
    }
    pub fn make_read_text(&self) -> Self {
        self.replace_url().remove_spoiler().replace_by_dict()
    }
}
