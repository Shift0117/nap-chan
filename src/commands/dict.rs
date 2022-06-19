use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, Seek, Write},
    sync::Arc,
};

use serde_json::to_string;
use serenity::{
    client::Context,
    model::{
        id::{ChannelId, UserId},
        interactions::application_command::ApplicationCommandInteraction,
    },
    prelude::TypeMapKey,
};
use tokio::sync::Mutex;
type SlashCommandResult = Result<String, String>;
pub struct DictHandler;

#[derive(Debug)]
pub struct State {
    pub dict: HashMap<String, String>,
    pub greeting_dict: HashMap<UserId, HashMap<String, String>>,
    pub voice_type_dict: HashMap<UserId, u8>,
    pub read_channel: Option<ChannelId>,
}

impl State {
    pub fn get_greeting(&self, user_id: &UserId, kinds: &str) -> Option<String> {
        Some(self.greeting_dict.get(user_id)?.get(kinds)?.clone())
    }
}

impl TypeMapKey for DictHandler {
    type Value = Arc<Mutex<State>>;
}
use crate::lib::text::{DICT_PATH, GREETING_DICT_PATH, VOICE_TYPE_DICT_PATH};

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
    Ok(format!("これからは、{}を{}って読むね", before, after))
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

pub async fn bye(
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
        .insert("bye".to_string(), greet.to_string());
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

pub fn create_empty_json(path: &str) -> File {
    let mut tmp = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(path)
        .expect("File creation error");
    tmp.write_all("{}".as_bytes()).ok();
    tmp.seek(std::io::SeekFrom::Start(0)).ok();
    tmp
}

pub fn generate_dictonaries() -> State {
    let dict_file = if let Ok(file) = std::fs::File::open(DICT_PATH) {
        file
    } else {
        create_empty_json(DICT_PATH)
    };
    let greeting_dict_file = if let Ok(file) = std::fs::File::open(GREETING_DICT_PATH) {
        file
    } else {
        create_empty_json(GREETING_DICT_PATH)
    };
    let voice_type_dict_file = if let Ok(file) = std::fs::File::open(VOICE_TYPE_DICT_PATH) {
        file
    } else {
        create_empty_json(VOICE_TYPE_DICT_PATH)
    };

    let reader = std::io::BufReader::new(dict_file);
    let dict: HashMap<String, String> = serde_json::from_reader(reader).expect("JSON parse error");

    let greeting_reader = std::io::BufReader::new(greeting_dict_file);
    let greeting_dict: HashMap<UserId, HashMap<String, String>> =
        serde_json::from_reader(greeting_reader).expect("JSON parse error");

    let voice_type_reader = BufReader::new(voice_type_dict_file);
    let voice_type_dict: HashMap<UserId, u8> =
        serde_json::from_reader(voice_type_reader).expect("JSON parse error");
    State {
        dict,
        greeting_dict,
        voice_type_dict,
        read_channel: None,
    }
}

pub async fn set_voice_type(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    voice_type: u8,
) -> SlashCommandResult {
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let voice_type_dict = &mut dict_lock.lock().await.voice_type_dict;
    let author_id = command.member.as_ref().unwrap().user.id;
    voice_type_dict.insert(author_id, voice_type);
    let voice_type_dict_json = to_string(&voice_type_dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(VOICE_TYPE_DICT_PATH)
        .unwrap();
    dict_file
        .write_all(voice_type_dict_json.as_bytes())
        .unwrap();
    dict_file.flush().unwrap();
    Ok(format!("ボイスタイプを{}に変えたよ", voice_type).to_string())
}
