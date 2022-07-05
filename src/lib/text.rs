use regex;
use serenity::async_trait;

use super::db::DictDB;
use alkana_rs::ALKANA;
use wana_kana::{is_katakana::is_katakana, to_katakana::to_katakana};

#[async_trait]
pub trait TextMessage {
    fn replace_url(&self) -> Self;
    fn remove_spoiler(&self) -> Self;
    async fn replace_by_dict(&self, database: &sqlx::SqlitePool) -> Self;
    fn remove_custom_emoji(&self) -> Self;
    async fn make_read_text(&self, database: &sqlx::SqlitePool) -> Self;
    fn hiraganize(&self) -> Self;
}
#[async_trait]
impl TextMessage for String {
    fn replace_url(&self) -> Self {
        let re = regex::Regex::new(r"https?://[\w!?/+\-_~;.,*&@#$%()='\[\]]+").unwrap();
        re.replace_all(self, "URL").to_string()
    }
    fn remove_spoiler(&self) -> Self {
        let re = regex::Regex::new(r"\|\|.+?\|\|").unwrap();
        re.replace_all(self, "").to_string()
    }
    async fn replace_by_dict(&self, database: &sqlx::SqlitePool) -> Self {
        let mut text = self.clone();
        for w in database.get_dict_all().await {
            tracing::info!("{:?} {:?}", &w, &text);
            let before = &w.word;
            let after = &w.read_word;
            text = text.replace(before, after);
        }
        text
    }
    fn hiraganize(&self) -> Self {
        let re = regex::Regex::new(r"[a-zA-Z]+").unwrap();
        let mut text = self.clone();
        for c in re.captures_iter(self) {
            if let Some(english_match) = c.get(0) {
                let english = english_match.as_str();
                let result = ALKANA.get_katakana(english);
                let mut temp = english.to_string();
                if let Some(result) = result {
                    temp = result;
                } else {
                    let katakana = to_katakana(english);
                    if is_katakana(&katakana) {
                        temp = katakana;
                    } else {
                        if let Some(words) = min_split(english) {
                            for word in words.iter() {
                                temp = temp.replacen(word, &ALKANA.get_katakana(&word).unwrap(), 1);
                            }
                        }
                    }
                }
                text = text.replacen(english, &temp, 1);
            }
        }
        text
    }
    fn remove_custom_emoji(&self) -> Self {
        let re = regex::Regex::new(r"<@.+?>").unwrap();
        re.replace_all(self, "").to_string()
    }
    async fn make_read_text(&self, database: &sqlx::SqlitePool) -> Self {
        self.replace_url()
            .remove_spoiler()
            .remove_custom_emoji()
            .replace_by_dict(database)
            .await
            .hiraganize()
    }
}

#[test]
fn hiraganize_test() {
    let word = "hello".to_string();
    assert_eq!("ハロー".to_string(), word.hiraganize());

    let sentence = "hello world".to_string();
    assert_eq!("ハロー ワールドゥ".to_string(), sentence.hiraganize());

    let hiragana = "はろーわーるど".to_string();
    assert_eq!("はろーわーるど".to_string(), hiragana.hiraganize());

    let mixed = "hello てすと world".to_string();
    assert_eq!("ハロー てすと ワールドゥ".to_string(), mixed.hiraganize());

    let romaji = "honyaraka".to_string();
    assert_eq!("ホニャラカ", romaji.hiraganize());

    let unknown = "sfhsakhba".to_string();
    assert_eq!(unknown, unknown.hiraganize());

    let jukugo = "firefox".to_string(); // fire,fox は辞書にあるが firefox はない
    assert_eq!("ファイアーフォックス", jukugo.hiraganize());
}

#[test]
fn alkana_speed_test() {
    let word = "hello";
    let now = std::time::Instant::now();
    for _ in 0..1000000 {
        ALKANA.get_katakana(word).unwrap();
    }
    println!("{}", now.elapsed().as_millis());
}

// 次の条件 (*) を満たすような str の分割 str = S_1 + S_2 + ... + S_k であって、k が最小であるものを求める
// (*) S_1,S_2,...,S_k はすべて alkana で変換可能
fn min_split(str: &str) -> Option<Vec<String>> {
    let str = str.chars().collect::<Vec<_>>();
    let n = str.len();
    let mut table = vec![vec![false; n]; n];
    // table[i][j] := str[i..=j] が変換可能かどうか
    let mut dp: Vec<usize> = vec![n + 1; n + 1];
    dp[0] = 0;
    // dp[i] := S[0..i] に対しての問題の結果
    // dp[i] = min_{0 <= j < i,table[j..i] = true}{dp[j]} + 1
    for i in 0..n {
        for j in i + 1..n {
            table[i][j] = ALKANA
                .get_katakana(&str[i..=j].iter().collect::<String>())
                .is_some();
        }
    }
    for i in 1..=n {
        dp[i] = (0..i)
            .filter(|j| table[*j][i - 1])
            .map(|j| dp[j])
            .min()
            .unwrap_or(n)
            + 1;
    }
    let k = dp[n];
    if k > n {
        None
    } else {
        let mut ans = Vec::new();
        let mut cur = k;
        let mut prev_idx = n;
        for i in (0..n).rev() {
            if dp[i] + 1 == cur && table[i][prev_idx-1] {
                ans.push(str[i..prev_idx].iter().collect::<String>());
                prev_idx = i;
                cur -= 1;
            }
        }
        ans.reverse();
        Some(ans)
    }
}

#[test]
fn min_split_test() {
    let str = "firefoxfoxfoxoxford";
    assert_eq!(min_split(str),Some(vec!["fire".to_string(),"fox".to_string(),"fox".to_string(),"fox".to_string(),"oxford".to_string()]));

    let unknown = "fssjkfsahfkajsh";
    assert_eq!(min_split(unknown),None);

    let str = "flightsimulator";
    dbg!(min_split(str));
}
