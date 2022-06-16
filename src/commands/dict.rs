use std::{io::Write};

use serde_json::to_string;
use serenity::{
    client::Context, framework::standard::CommandResult,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::lib::text::{DictHandler, DICT_PATH};

pub async fn add(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    before: &str,
    after: &str,
) -> CommandResult {
    dbg!(&before, &after);
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dict = dict_lock.lock().await;
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

    Ok(())
}

pub async fn rem(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    word: &str,
) -> CommandResult {
    let dict_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<DictHandler>().unwrap().clone()
    };
    let mut dict = dict_lock.lock().await;
    if dict.contains_key(word) {
        dict.remove(word);
    }
    let dict = dict.clone();
    let dict_json = to_string(&dict).unwrap();
    let mut dict_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open("read_dict.json")
        .unwrap();
    dict_file.write_all(dict_json.as_bytes()).unwrap();
    dict_file.flush().unwrap();
    Ok(())
}
