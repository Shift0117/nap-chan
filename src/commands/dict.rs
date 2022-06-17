use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Seek, Write},
    sync::Arc,
};

use serde_json::to_string;
use serenity::{
    client::Context,
    model::{id::UserId, interactions::application_command::ApplicationCommandInteraction},
    prelude::TypeMapKey,
};
use tokio::sync::Mutex;
type SlashCommandResult = Result<String, String>;
pub struct DictHandler;

#[derive(Debug)]
pub struct Dicts {
    pub dict: HashMap<String, String>,
    pub greeting_dict: HashMap<UserId, HashMap<String, String>>,
}

impl Dicts {
    pub fn get_greeting(&self, user_id: &UserId, kinds: &str) -> Option<String> {
        Some(self.greeting_dict.get(user_id)?.get(kinds)?.clone())
    }
}

impl TypeMapKey for DictHandler {
    type Value = Arc<Mutex<Dicts>>;
}
use crate::lib::text::{DICT_PATH, GREETING_DICT_PATH};

pub async fn add(
    ctx: &Context,
    _command: &ApplicationCommandInteraction,
    before: &str,
    after: &str,
) -> SlashCommandResult {
    dbg!(&before, &after);
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dicts = dict_lock.lock().await;
    let dict = &mut dicts.dict;
    dict.insert(before.to_string(), after.to_string());
    let dict = dict.clone();
    let dict_json = to_string(&dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(DICT_PATH)
        .unwrap();
    dict_file.write_all(dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();
    Ok(format!("これからは{}って読むね", after))
}

pub async fn rem(
    ctx: &Context,
    _command: &ApplicationCommandInteraction,
    word: &str,
) -> SlashCommandResult {
    let dicts_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dicts = dicts_lock.lock().await;
    if dicts.dict.contains_key(word) {
        dicts.dict.remove(word);
    }
    let dict = dicts.dict.clone();
    let dict_json = to_string(&dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open("read_dict.json")
        .unwrap();
    dict_file.write_all(dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();
    Ok(format!("これからは{}って読むね", word))
}

pub async fn hello(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    greet: &str,
) -> SlashCommandResult {
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dicts = dict_lock.lock().await;
    let greeting_dict = &mut dicts.greeting_dict;
    let author_id = command.member.as_ref().unwrap().user.id;
    greeting_dict
        .entry(author_id)
        .or_insert(HashMap::new())
        .insert("hello".to_string(), greet.to_string());
    let greeting_dict_json = to_string(&greeting_dict).unwrap();
    dbg!(&greeting_dict_json);
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(GREETING_DICT_PATH)
        .unwrap();
    dict_file.write_all(greeting_dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();
    Ok(format!(
        "{}さん、これから{}ってあいさつするね",
        command.member.as_ref().unwrap().user.name,
        greet
    ))
}

pub fn generate_dictonaries() -> Dicts {
    let dict_file = if let Ok(file) = std::fs::File::open(DICT_PATH) {
        file
    } else {
        let mut tmp = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(DICT_PATH)
            .expect("File creation error");
        tmp.write_all("{}".as_bytes()).ok();
        tmp.seek(std::io::SeekFrom::Start(0)).ok();

        tmp
    };
    let greeting_dict_file = std::fs::File::open(GREETING_DICT_PATH).unwrap_or({
        let mut tmp = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(GREETING_DICT_PATH)
            .expect("File creation error");
        tmp.write_all("{}\n".as_bytes()).ok();
        tmp.seek(std::io::SeekFrom::Start(0)).ok();
        tmp
    });
    let reader = std::io::BufReader::new(dict_file);
    let dict: HashMap<String, String> = serde_json::from_reader(reader).expect("JSON parse error");

    let greeting_reader = std::io::BufReader::new(greeting_dict_file);
    let greeting_dict: HashMap<UserId, HashMap<String, String>> =
        serde_json::from_reader(greeting_reader).unwrap();
    Dicts {
        dict,
        greeting_dict,
    }
}
