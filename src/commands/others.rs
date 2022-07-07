use anyhow::{anyhow, Result};
use std::{
    fmt::Debug,
    fs::File,
    io::{Cursor, Write},
    str::Bytes,
};

pub async fn simple_wolfram_alpha(input: &str) -> Result<String> {
    dotenv::dotenv().ok();
    let url = "http://api.wolframalpha.com/v2/simple";
    let app_id = std::env::var("WOLFRAM_ALPHA_APP_ID")?;

    let params = [("i", input), ("appid", &app_id)];
    let client = reqwest::Client::new();
    let res = client.get(url).query(&params).send().await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let path = format!("temp/{}.gif", now);
    let mut file = std::fs::File::create(&path)?;
    file.write(&res.bytes().await?).ok();
    Ok(path)
}
