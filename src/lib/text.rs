use regex;

struct Text {
    text: String,
}
impl Text {
    pub fn new(text:String) -> Self {
        Text { text }
    }
    fn replace_url(&self)-> Self {
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='[\]]+").unwrap();
        Self::new(re.replace(&self.text, "URL").to_string())
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        Self::new(re.replace(&self.text, "").to_string())
    }
    fn replace_by_dict(&self) -> Self {
        todo!()
    }
    pub fn make_read_text(&self) -> Self {
        self.replace_url().remove_spoiler().replace_by_dict()
    }
}
