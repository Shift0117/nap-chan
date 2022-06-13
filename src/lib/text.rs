<<<<<<< HEAD
use regex;

struct Text {
    text: String,
=======
use std::{fs::File, io::BufReader, collections::HashMap};

use regex;
use serde_json;

#[derive(Debug,Clone)]
pub struct Text {
    pub text: String,
>>>>>>> 2033ebb016491ad744ef26220606b3a19d3e711a
}
impl Text {
    pub fn new(text:String) -> Self {
        Text { text }
    }
    fn replace_url(&self)-> Self {
<<<<<<< HEAD
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='[\]]+").unwrap();
=======
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='\[\]]+").unwrap();
>>>>>>> 2033ebb016491ad744ef26220606b3a19d3e711a
        Self::new(re.replace(&self.text, "URL").to_string())
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        Self::new(re.replace(&self.text, "").to_string())
    }
    fn replace_by_dict(&self) -> Self {
<<<<<<< HEAD
        todo!()
=======
        let mut text = self.text.clone();
        let dict_file = File::open("read_dict.json").unwrap();
        let reader = BufReader::new(dict_file);
        let map:Vec<(String,String)> = serde_json::from_reader(reader).unwrap();
        for (k,v) in map.iter() {
            text = text.replace(k, v);
        }
        Self::new(text)
>>>>>>> 2033ebb016491ad744ef26220606b3a19d3e711a
    }
    pub fn make_read_text(&self) -> Self {
        self.replace_url().remove_spoiler().replace_by_dict()
    }
}
