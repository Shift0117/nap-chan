use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use std::{
    fmt::Debug,
    fs::File,
    io::{Cursor, Write},
};

pub async fn simple_wolfram_alpha(input: &str) -> Result<String> {
    dotenv::dotenv().ok();
    let url = "http://api.wolframalpha.com/v2/result";
    let app_id = std::env::var("WOLFRAM_ALPHA_APP_ID")?;

    let params = [("i", input), ("appid", &app_id)];
    let client = reqwest::Client::new();
    let res = client.get(url).query(&params).send().await?;
    let result = res.text().await?;
    Ok(result)
}
