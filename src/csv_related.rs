use grammers_client::Client;
use grammers_client::types::Chat;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::time::{Duration, sleep};

#[derive(Serialize, Deserialize)]
struct MyUser {
    id: i64,
    full_name: String,
    tg_username: Option<String>,
}
impl MyUser {
    fn get_mention(&self) -> String {
        return match &self.tg_username {
            None => format!("@{}({})", self.id, &self.full_name),
            Some(username) => format!("@{}", &username),
        };
    }
}

pub fn filepath_from_chat(chat: &Chat) -> String {
    return format!("csv/{chat_id}.csv", chat_id = chat.id()); // TODO: refactor into Path or sim.
}

pub async fn generate_csv(client: &Client, chat: &Chat) -> Result<(), Box<dyn Error>> {
    // 4. Export participants to CSV
    let csv_filepath = filepath_from_chat(chat);
    debug!("Generating {path}", path = &csv_filepath);
    let mut wtr = csv::Writer::from_path(&csv_filepath)?;

    // the csv header is used automatically when serializing from a struct
    let mut iter = client.iter_participants(chat);
    while let Some(part) = iter.next().await? {
        let user = &part.user;
        let my_user = MyUser {
            id: user.id(),
            full_name: user.full_name(),
            tg_username: user.username().map(String::from),
        };
        wtr.serialize(my_user)?;
        sleep(Duration::from_millis(25)).await;
    }
    wtr.flush()?;
    info!("Exported participants to {path}", path = &csv_filepath);

    return Ok(());
}

pub fn parse_pings_of_chat(chat: &Chat) -> Result<String, Box<dyn Error>> {
    // 5. Read CSV and ping each user
    let filepath = filepath_from_chat(chat);
    let mut rdr = csv::Reader::from_path(&filepath)?;
    let iter = rdr.deserialize();
    let bot_mention = std::env::var("BOT_MENTION").unwrap_or_default();

    debug!("Parsing {path}", path = &filepath);
    let mut result_message = String::new(); // the final message with space-separated mentions
    for result in iter {
        let user: MyUser = result?;
        let mention = user.get_mention();
        if mention == bot_mention {
            continue;
        }
        result_message.push_str(&mention);
        result_message.push(' ');
    }

    return Ok(result_message);
}
